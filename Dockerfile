# =============================================================================
# SAF Docker build — multi-stage (Ubuntu 24.04 LTS)
# =============================================================================

# -----------------------------------------------------------------------------
# Stage: base — shared dependencies
# -----------------------------------------------------------------------------
FROM ubuntu:24.04 AS base

ENV DEBIAN_FRONTEND=noninteractive

# APT mirror override. Set APT_MIRROR in your .env or docker-compose to use
# a faster mirror for your region (e.g., mirrors.aliyun.com for China,
# mirrors.ustc.edu.cn for USTC). Leave empty for default Ubuntu mirrors.
ARG APT_MIRROR=
RUN if [ -n "$APT_MIRROR" ]; then \
      sed -i "s|ports.ubuntu.com|${APT_MIRROR}|g; s|archive.ubuntu.com|${APT_MIRROR}|g" \
        /etc/apt/sources.list.d/ubuntu.sources; \
    fi

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    ca-certificates \
    cmake \
    curl \
    git \
    pkg-config \
    llvm-18-dev \
    libclang-18-dev \
    clang-18 \
    libpolly-18-dev \
    libzstd-dev \
    mold \
    python3.12 \
    python3.12-dev \
    python3.12-venv \
    python3-pip \
    && rm -rf /var/lib/apt/lists/*

# Make python3 point to 3.12
RUN update-alternatives --install /usr/bin/python3 python3 /usr/bin/python3.12 1

# LLVM config for inkwell/llvm-sys (version 181.x for LLVM 18.1)
ENV LLVM_SYS_181_PREFIX=/usr/lib/llvm-18

# Install Rust toolchain to shared location (accessible by any UID)
ENV RUSTUP_HOME=/opt/rustup
ENV CARGO_HOME=/opt/cargo
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- -y --default-toolchain 1.85.0
ENV PATH="/opt/cargo/bin:${PATH}"

# Install cargo-nextest for parallel test execution (pinned for rustc 1.85 compat)
RUN cargo install cargo-nextest@0.9.100 --locked

# Make cargo/rustup world-writable so non-root users can install targets/components
RUN chmod -R a+rwX /opt/cargo /opt/rustup

# Use mold linker for faster linking (Docker-only)
ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER="clang-18"
ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C link-arg=-fuse-ld=mold"
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER="clang-18"
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C link-arg=-fuse-ld=mold"

# Install maturin
RUN pip3 install --break-system-packages maturin>=1.7

WORKDIR /workspace

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

# Create virtualenv for maturin develop
RUN python3 -m venv /workspace/.venv
ENV VIRTUAL_ENV=/workspace/.venv
ENV PATH="/workspace/.venv/bin:$PATH"

RUN pip install pytest>=8.0 pytest-cov>=5.0

COPY . /workspace

# Build Rust workspace (excluding saf-python which needs maturin)
RUN cargo build --workspace --exclude saf-python

# Build + install the Python extension
RUN maturin develop

# Run all tests
CMD ["sh", "-c", "cargo nextest run --workspace --exclude saf-python && pytest python/tests/ -v"]

# -----------------------------------------------------------------------------
# Stage: runtime — minimal image with saf binary
# -----------------------------------------------------------------------------
FROM ubuntu:24.04 AS runtime

ARG APT_MIRROR=
RUN if [ -n "$APT_MIRROR" ]; then \
      sed -i "s|ports.ubuntu.com|${APT_MIRROR}|g; s|archive.ubuntu.com|${APT_MIRROR}|g" \
        /etc/apt/sources.list.d/ubuntu.sources; \
    fi

RUN apt-get update && apt-get install -y --no-install-recommends \
    libllvm18 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=test /workspace/target/debug/saf /usr/local/bin/saf

# Copy function specs for external/library function modeling
COPY --from=test /workspace/share/saf/specs /usr/local/share/saf/specs

ENTRYPOINT ["saf"]
