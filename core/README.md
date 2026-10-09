# Gem Wallet Core

Shared Rust libraries, mobile bindings, and backend services for [Gem Wallet](../README.md). See the root README for app setup, contributing, security reporting, and community links.

## Layout

- [`crates/`](crates/): shared models, blockchain implementations, signing, swaps, storage, and provider integrations
- [`gemstone/`](gemstone/): UniFFI library consumed by the iOS and Android apps
- [`apps/`](apps/): API, daemon, and Dynode backend services
- [`bin/`](bin/): command-line tools and code generators

See the [Core features and providers comparison](../docs/FEATURES.md) for supported capabilities.

## Development

Run the following commands from `core/`. First-time prerequisites and build cache configuration are in [Setup](skills/setup.md).

```sh
just build
just test
just test primitives
just format
just lint
just audit
```

`just test <CRATE>` runs a specific crate's unit tests. `just audit` checks dependencies for known vulnerabilities and installs `cargo-audit` if needed. See [Development Commands](skills/development-commands.md) for integration tests and other recipes.

## Running the API

Install the PostgreSQL client libraries and Diesel CLI described in [Setup](skills/setup.md). With Docker available, start the local backend dependencies:

```sh
just setup-services
```

This starts PostgreSQL, Redis, Meilisearch, and RabbitMQ using [docker-compose.yml](docker-compose.yml). Configure `DATABASE_URL` for Diesel to point to the local database, then run:

```sh
just migrate
cargo run --package api
```

The API reads [Settings.yaml](Settings.yaml) from the working directory, with environment variable overrides such as `POSTGRES_URL` and `REDIS_URL`. Configure service endpoints and any required provider credentials for your environment before starting it.

## Mobile Integration

Gemstone exposes Core functionality to Swift and Kotlin through UniFFI. Follow the [root README](../README.md) to build the apps and [Development Commands](skills/development-commands.md#generating-bindings-when-core-changes-affect-mobile-apis) for binding generation and standalone Gemstone examples.

## License

Gem Wallet Core is licensed under the [MIT License](LICENSE).
