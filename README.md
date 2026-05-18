# Driver API

This repository provides the Driver API, which allows truck drivers to manage their workdays, breaks, etc.

## Prerequisites

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

#### Available commands

**`delete-garbage`** — Deletes expired workday garbage entries. Intended to run periodically (e.g. daily).

```bash
cargo run -p job -- delete-garbage
```

**`generate-documents`** — Generates workday documents for all (driver, month) pairs older than N months that have at least one workday and no document yet.

```bash
cargo run -p job -- generate-documents --months-ago 3
```

#### Configuration

All options can be set via environment variables or CLI flags (CLI flags take precedence).

| Environment variable          | CLI flag                        | Default                                                |
| ----------------------------- | ------------------------------- | ------------------------------------------------------ |
| `DATABASE_URL`                | `--database-url`                | `postgres://postgres:password@localhost:5432/plannify` |
| `REDIS_URL`                   | `--redis-url`                   | `redis://localhost:6379/0`                             |
| `FRONTEND_URL`                | `--frontend-url`                | `https://app.plannify.be`                              |
| `PDF_SERVICE_ENDPOINT`        | `--pdf-service-endpoint`        | `http://localhost:50051`                               |
| `SMTP_DEFAULT_SENDER`         | `--smtp-default-sender`         | `noreply@plannify.be`                                  |
| `SMTP_USERNAME`               | `--smtp-username`               | _(empty)_                                              |
| `SMTP_PASSWORD`               | `--smtp-password`               | _(empty)_                                              |
| `SMTP_DOMAIN`                 | `--smtp-domain`                 | `localhost`                                            |
| `S3_ACCESS_KEY`               | `--s3-access-key`               | _(empty)_                                              |
| `S3_SECRET_KEY`               | `--s3-secret-key`               | _(empty)_                                              |
| `S3_ENDPOINT`                 | `--s3-endpoint`                 | `http://localhost:3900`                                |
| `S3_BUCKET_NAME`              | `--s3-bucket-name`              | `plannify`                                             |
| `S3_REGION`                   | `--s3-region`                   | `garage`                                               |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | `--otel-exporter-otlp-endpoint` | `http://localhost:4317`                                |
| `OTEL_SERVICE_NAME`           | `--otel-service-name`           | `driver-job`                                           |

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
