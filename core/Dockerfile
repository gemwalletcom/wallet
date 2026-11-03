FROM mirror.gcr.io/library/rust:1.99.0-trixie@sha256:81015672d6939b0265f09adf96d5de125482bb1330106076956b0518e74edc90 AS chef
RUN cargo install --locked cargo-chef --version 0.1.77
WORKDIR /app
ENV CARGO_INCREMENTAL=0 \
    CARGO_TERM_COLOR=always

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder-core
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json \
    --package api --package daemon
COPY . .
RUN cargo build --release --package api --package daemon && cp target/release/api target/release/daemon /app/

FROM chef AS builder-dynode
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json \
    --package dynode
COPY . .
RUN cargo build --release --package dynode && cp target/release/dynode /app/

# Shared runtime base
FROM mirror.gcr.io/library/debian:trixie-slim@sha256:f6b08b09a05725f9431502d3b5714f5c4576e82c1a84fa1debe0c24a7e0c3ec4 AS runtime-base
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y --no-install-recommends openssl ca-certificates && rm -rf /var/lib/apt/lists/*

# Core runtime image
FROM runtime-base AS core
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder-core /app/api /app/
COPY --from=builder-core /app/daemon /app/
COPY --from=builder-core /app/Settings.yaml /app/
CMD ["sh", "-c", "/app/${BINARY}"]

# Dynode runtime image
FROM runtime-base AS dynode
WORKDIR /app
COPY --from=builder-dynode /app/dynode /app/
COPY --from=builder-dynode /app/apps/dynode/config.yml /app/
CMD ["/app/dynode"]
