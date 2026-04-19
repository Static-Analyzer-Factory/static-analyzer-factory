# =============================================================================
# SAF Docker build — multi-stage (Ubuntu 24.04 LTS)
#
# LLVM version is selectable via ARG LLVM_VERSION:
#   - 18 (default): uses Ubuntu 24.04 distro packages
#   - 22: uses apt.llvm.org (Ubuntu distro does not ship llvm-22)
#
# Example:
#   docker build --build-arg LLVM_VERSION=18 -t saf:llvm18 .
#   docker build --build-arg LLVM_VERSION=22 -t saf:llvm22 .
# =============================================================================

# -----------------------------------------------------------------------------
# Stage: base — shared dependencies
# -----------------------------------------------------------------------------
FROM ubuntu:24.04 AS base

ENV DEBIAN_FRONTEND=noninteractive

# APT mirror override. Set to "" to keep the default Ubuntu mirrors.
# Default: mirrors.aliyun.com (fast in China; the stock ports/archive mirrors
# are extremely slow from many networks).
ARG APT_MIRROR=mirrors.aliyun.com
RUN if [ -n "$APT_MIRROR" ]; then \
      sed -i "s|ports.ubuntu.com|${APT_MIRROR}|g; s|archive.ubuntu.com|${APT_MIRROR}|g" \
        /etc/apt/sources.list.d/ubuntu.sources; \
    fi

# Which LLVM toolchain to install. "18" uses Ubuntu distro packages;
# anything else adds the apt.llvm.org repository first.
ARG LLVM_VERSION=18
ENV LLVM_VERSION=${LLVM_VERSION}

# Install the common apt prerequisites, then optionally add apt.llvm.org for
# LLVM versions not in the distro, then install the LLVM toolchain.
RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates \
        curl \
        gnupg \
        wget \
    && if [ "$LLVM_VERSION" != "18" ]; then \
         wget -qO - https://apt.llvm.org/llvm-snapshot.gpg.key \
           | gpg --dearmor > /etc/apt/trusted.gpg.d/apt.llvm.org.gpg && \
         echo "deb https://apt.llvm.org/noble/ llvm-toolchain-noble-${LLVM_VERSION} main" \
           > /etc/apt/sources.list.d/llvm.list && \
         apt-get update; \
       fi \
    && apt-get install -y --no-install-recommends \
        build-essential \
        cmake \
        git \
        pkg-config \
        "llvm-${LLVM_VERSION}-dev" \
        "libclang-${LLVM_VERSION}-dev" \
        "clang-${LLVM_VERSION}" \
        "libpolly-${LLVM_VERSION}-dev" \
        libzstd-dev \
        mold \
        python3.12 \
        python3.12-dev \
        python3.12-venv \
        python3-pip \
    && rm -rf /var/lib/apt/lists/*

# Make python3 point to 3.12
RUN update-alternatives --install /usr/bin/python3 python3 /usr/bin/python3.12 1

# llvm-sys prefix — the build script for llvm-sys-<major> reads the env var
# matching its major/minor (e.g., LLVM_SYS_181_PREFIX for LLVM 18.1). Only the
# one matching the active cargo feature is actually consulted, so setting both
# variants is harmless on an image that only has one LLVM installed.
ENV LLVM_SYS_181_PREFIX=/usr/lib/llvm-18
ENV LLVM_SYS_221_PREFIX=/usr/lib/llvm-22

# Install Rust toolchain to shared location (accessible by any UID)
ENV RUSTUP_HOME=/opt/rustup
ENV CARGO_HOME=/opt/cargo
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- -y --default-toolchain 1.88.0
ENV PATH="/opt/cargo/bin:${PATH}"

# Install cargo-nextest for parallel test execution (pinned for rustc 1.88+ compat)
RUN cargo install cargo-nextest@0.9.100 --locked

# Make cargo/rustup world-writable so non-root users can install targets/components
RUN chmod -R a+rwX /opt/cargo /opt/rustup

# Use mold linker via the selected clang (faster linking, Docker-only)
ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER="clang-${LLVM_VERSION}"
ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C link-arg=-fuse-ld=mold"
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER="clang-${LLVM_VERSION}"
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C link-arg=-fuse-ld=mold"

# Install maturin
RUN pip3 install --break-system-packages "maturin>=1.7"

WORKDIR /workspace

# Pre-create volume mount points with permissive permissions so that
# Docker named volumes (which are root-owned on first creation) are
# writable by the non-root UID specified in docker-compose.yml.
RUN mkdir -p /workspace/target /workspace/target-maturin && \
    chmod 777 /workspace/target /workspace/target-maturin

# -----------------------------------------------------------------------------
# Stage: dev — interactive development shell
# -----------------------------------------------------------------------------
FROM base AS dev

COPY docker/dev-entrypoint.sh /usr/local/bin/dev-entrypoint.sh
RUN chmod +x /usr/local/bin/dev-entrypoint.sh

# Mount source via docker-compose volume; nothing to COPY here.
ENTRYPOINT ["dev-entrypoint.sh"]
CMD ["bash"]

# -----------------------------------------------------------------------------
# Stage: test — reproducible test run from copied source
# -----------------------------------------------------------------------------
FROM base AS test

# ARG is not inherited across FROM stages; re-declare.
ARG LLVM_VERSION=18
ENV LLVM_VERSION=${LLVM_VERSION}

# Create virtualenv for maturin develop
RUN python3 -m venv /workspace/.venv
ENV VIRTUAL_ENV=/workspace/.venv
ENV PATH="/workspace/.venv/bin:$PATH"

RUN pip install "pytest>=8.0" "pytest-cov>=5.0"

COPY . /workspace

# Build Rust workspace (excluding saf-python which needs maturin). For LLVM 18
# we use defaults; for other versions we disable defaults and explicitly
# re-enable the features downstream crates normally pull in.
RUN if [ "$LLVM_VERSION" = "18" ]; then \
      cargo build --workspace --exclude saf-python; \
    else \
      cargo build --workspace --exclude saf-python --no-default-features \
        --features "saf-frontends/llvm-${LLVM_VERSION},saf-analysis/z3-solver,saf-core/logging-subscriber,saf-cli/llvm-${LLVM_VERSION},saf-bench/llvm-${LLVM_VERSION},saf-trace/llvm-${LLVM_VERSION},saf-datalog/llvm-${LLVM_VERSION},saf-test-utils/llvm-${LLVM_VERSION}"; \
    fi

# Build + install the Python extension
RUN if [ "$LLVM_VERSION" = "18" ]; then \
      maturin develop; \
    else \
      maturin develop --no-default-features --features "llvm-${LLVM_VERSION}"; \
    fi

# Run all tests
CMD ["sh", "-c", "\
  if [ \"$LLVM_VERSION\" = \"18\" ]; then \
    cargo nextest run --workspace --exclude saf-python && pytest python/tests/ -v; \
  else \
    cargo nextest run --workspace --exclude saf-python --no-default-features \
      --features \"saf-frontends/llvm-${LLVM_VERSION},saf-analysis/z3-solver,saf-core/logging-subscriber,saf-cli/llvm-${LLVM_VERSION},saf-bench/llvm-${LLVM_VERSION},saf-trace/llvm-${LLVM_VERSION},saf-datalog/llvm-${LLVM_VERSION},saf-test-utils/llvm-${LLVM_VERSION}\" \
      && pytest python/tests/ -v; \
  fi"]

# -----------------------------------------------------------------------------
# Stage: runtime — minimal image with saf binary
# -----------------------------------------------------------------------------
FROM ubuntu:24.04 AS runtime

ARG APT_MIRROR=mirrors.aliyun.com
ARG LLVM_VERSION=18

RUN if [ -n "$APT_MIRROR" ]; then \
      sed -i "s|ports.ubuntu.com|${APT_MIRROR}|g; s|archive.ubuntu.com|${APT_MIRROR}|g" \
        /etc/apt/sources.list.d/ubuntu.sources; \
    fi

# Install libllvm matching the build's LLVM version. For versions not in the
# distro, add apt.llvm.org first.
RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates \
    && if [ "$LLVM_VERSION" != "18" ]; then \
         apt-get install -y --no-install-recommends gnupg wget && \
         wget -qO - https://apt.llvm.org/llvm-snapshot.gpg.key \
           | gpg --dearmor > /etc/apt/trusted.gpg.d/apt.llvm.org.gpg && \
         echo "deb https://apt.llvm.org/noble/ llvm-toolchain-noble-${LLVM_VERSION} main" \
           > /etc/apt/sources.list.d/llvm.list && \
         apt-get update; \
       fi \
    && apt-get install -y --no-install-recommends \
        "libllvm${LLVM_VERSION}" \
    && rm -rf /var/lib/apt/lists/*

COPY --from=test /workspace/target/debug/saf /usr/local/bin/saf

# Copy function specs for external/library function modeling
COPY --from=test /workspace/share/saf/specs /usr/local/share/saf/specs

ENTRYPOINT ["saf"]
