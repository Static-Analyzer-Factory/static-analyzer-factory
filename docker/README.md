# Docker Setup

SAF builds and tests run inside Docker (Ubuntu 24.04 LTS) because LLVM 18 is required.

## Quick Start

```bash
make shell       # Interactive dev shell
make test        # Run all tests (cached — fast)
make test-clean  # Run all tests (hermetic — clean build)
make lint        # Run clippy + rustfmt check
make build       # Build minimal runtime image
```

## Test Modes

### `make test` (default — cached)

Uses persistent Docker volumes to cache compiled dependencies (including z3-sys, which takes several minutes to compile from C++ source). After the first run, subsequent test runs only recompile changed SAF code.

### `make test-clean` (hermetic)

Builds everything from scratch inside the Docker image with no cache. Slower, but fully reproducible. Use this for CI or when you suspect cache corruption.

To reset the cached volumes (force a full rebuild on next `make test`):

```bash
make clean
```

## APT Mirror Configuration

By default, the Docker build uses **mirrors.aliyun.com** as the APT mirror. The stock Ubuntu mirrors (`ports.ubuntu.com` on ARM, `archive.ubuntu.com` on x86_64) are very slow from many networks.

### Use the default (Aliyun)

No configuration needed:

```bash
make test
```

### Use a different mirror

Set the `APT_MIRROR` environment variable:

```bash
# USTC mirror (China)
APT_MIRROR=mirrors.ustc.edu.cn make test

# Tsinghua mirror (China)
APT_MIRROR=mirrors.tuna.tsinghua.edu.cn make test
```

### Use the official Ubuntu mirrors (no override)

Set `APT_MIRROR` to an empty string:

```bash
APT_MIRROR="" make test
```

### Persist your choice

Create a `.env` file in the project root (gitignored):

```bash
echo 'APT_MIRROR=mirrors.ustc.edu.cn' > .env
```

Docker Compose reads `.env` automatically, so all `make` commands will use your mirror without extra flags.

### Direct docker build

When building without docker-compose:

```bash
# With Aliyun (default)
docker build .

# With a different mirror
docker build --build-arg APT_MIRROR=mirrors.ustc.edu.cn .

# With official Ubuntu mirrors
docker build --build-arg APT_MIRROR="" .
```
