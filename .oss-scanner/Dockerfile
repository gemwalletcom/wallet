# OSS Scanner image: builds the Core workspace and its tests for offline scanning.
FROM mirror.gcr.io/library/rust:1.100.0-trixie@sha256:e84068000361bf3d4a5ba2b7647f62893528b1eb9f847d7a23b493f5ce8ea254
ENV DEBIAN_FRONTEND=noninteractive \
    CARGO_TERM_COLOR=never \
    CARGO_INCREMENTAL=0
RUN apt-get update && apt-get install -y --no-install-recommends clang cmake pkg-config libpq-dev python3 \
    && rm -rf /var/lib/apt/lists/*

COPY . /src
WORKDIR /src/core

RUN rustup show active-toolchain && cargo --version

ARG CARGO_BUILD_JOBS=16

RUN cargo fetch --locked

RUN cargo build --locked --workspace --all-features \
    && cargo test --locked --workspace --all-features --no-run

RUN timeout 1200 cargo test --locked --workspace --all-features --lib --bins -- --skip integration_tests \
    || echo "WARNING: cargo test reported failures"

ENV CARGO_NET_OFFLINE=true
