# Driver API 1

This repository provides the Driver API, which allows truck drivers to manage their workdays, breaks, etc.

## Prerequisites

Install the pinned Rust toolchain and tools with [mise](https://mise.jdx.dev) (see `mise.toml`):

```bash
mise install
```

Set the required environment variables in the `.env` file.
```bash
cp .env.template .env
```

Up the docker compose from [Common services](https://github.com/plannify-truck-driver/common-services)

```bash
docker compose -f ./common-services/docker-compose.yaml up -d
```

Get S3 credentials from the `common-services/garage.env` file and set them in the `.env` file.

## Usage

### Running the API

```bash
RUST_LOG=info cargo run --bin api
```

### Running jobs

The `job` crate is a CLI runner for background tasks. It reads configuration from environment variables (or a `.env` file) and from CLI flags.

```bash
cargo run -p job -- <command> [options]
```

### Testing

There are unit and integration tests available. The integration tests require the common services to be running and a dataset to be seeded.

```bash
sh test.sh
cargo test
```

## Development

Before making a pull request, please ensure to generate the sqlx files for any database changes:

```bash
cargo sqlx prepare --workspace
```
