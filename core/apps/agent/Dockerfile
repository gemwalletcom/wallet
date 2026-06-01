# syntax=docker/dockerfile:1
FROM lukemathwalker/cargo-chef:latest-rust-1.95.0-trixie AS chef
WORKDIR /app
ENV CARGO_INCREMENTAL=0 \
    CARGO_TERM_COLOR=always

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json --package agent
COPY . .
RUN cargo build --release --package agent && cp target/release/agent /app/agent

FROM debian:trixie-slim AS runtime
ENV DEBIAN_FRONTEND=noninteractive \
    HOME=/home/agent \
    AGENT_DIR=/home/agent/app \
    FASTEMBED_CACHE_PATH=/home/agent/app/cache/fastembed
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates curl git openssl gnupg jq ripgrep \
    && rm -rf /var/lib/apt/lists/*
RUN curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg \
        | gpg --dearmor -o /usr/share/keyrings/githubcli-archive-keyring.gpg \
    && echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" \
        > /etc/apt/sources.list.d/github-cli.list \
    && apt-get update \
    && apt-get install -y --no-install-recommends gh \
    && rm -rf /var/lib/apt/lists/*
RUN useradd --create-home --uid 1000 --shell /bin/bash agent
USER agent
COPY --from=builder /app/agent /usr/local/bin/agent
WORKDIR /home/agent/app
COPY --chown=agent:agent apps/agent/Settings.yaml ./
COPY --chown=agent:agent apps/agent/context ./context
COPY --chown=agent:agent apps/agent/agents ./agents
RUN mkdir -p /home/agent/app/data /home/agent/app/cache

CMD ["agent"]
