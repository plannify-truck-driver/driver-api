# Driver API

This repository provides the Driver API, which allows truck drivers to manage their workdays, breaks, etc.

## Prerequisites

```bash
cp .env.template .env
```

Up the docker compose from [Common services](https://github.com/plannify-truck-driver/common-services)

## Usage

### Running the API

```bash
RUST_LOG=info cargo run --bin api
```

### Testing

There are unit and integration tests available. The integration tests require the common services to be running and a dataset to be seeded.

```bash
PGPASSWORD=plannify_password psql -h localhost -U plannify_user -d plannify_db -f config/test-dataset.sql
cargo test
```

## Development

Before making a pull request, please ensure to generate the sqlx files for any database changes:

```bash
cargo sqlx prepare --workspace
```
