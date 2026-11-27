FROM mirror.gcr.io/library/rust:1.100.0-trixie@sha256:e84068000361bf3d4a5ba2b7647f62893528b1eb9f847d7a23b493f5ce8ea254 AS chef
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
FROM mirror.gcr.io/library/debian:trixie-slim@sha256:8aa5a7bf5a1955bddbe7d74fcdd0b91ab30e74362a8b73af672b0481b35b44a0 AS runtime-base
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
