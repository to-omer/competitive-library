# syntax=docker/dockerfile:1

# Rust toolchain (set via --build-arg RUST_VERSION, e.g. 1.89.0)
ARG RUST_VERSION=1.89.0
FROM rust:${RUST_VERSION}-bookworm

# Runtime UID/GID to avoid root-owned artifacts on host
ARG UID=1000
ARG GID=1000

# Toolchain & deps in one layer (for verify/library-checker builds).
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
    python3 python3-pip make g++ git ca-certificates \
    && cargo install --locked cargo-make codesnip \
    && rustup component add rustfmt clippy \
    && rm -rf /var/lib/apt/lists/*

# Create unprivileged user that matches host uid/gid and pre-own cache dirs
RUN groupadd -g ${GID} rustdev \
    && useradd -m -u ${UID} -g rustdev -s /bin/bash rustdev \
    && mkdir -p /tmp/target /cache \
    && chown -R rustdev:rustdev /usr/local/cargo /usr/local/rustup /tmp/target /cache

# Ensure initial named volumes inherit ownership
VOLUME ["/tmp/target", "/cache"]

WORKDIR /workspace

USER rustdev

CMD ["bash"]
