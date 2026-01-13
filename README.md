# Driver API

This repository provides the Driver API, which allows truck drivers to manage their workdays, breaks, etc.

## Prerequisites

```bash
cp .env.example .env
```

Up the docker compose from [Common services](https://github.com/plannify-truck-driver/common-services)

## Usage

### Running the API

```bash
RUST_LOG=info cargo run --bin api
```

### Testing

```bash
cargo test
```
