# OSS Scanner image: builds the Core workspace and its tests for offline scanning.
FROM mirror.gcr.io/library/rust:1.100.0-trixie@sha256:223cc32ddca8f3ee90c436dbc94b6b5f052ea7f9653c5201e9612f08ad7d21b8
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
