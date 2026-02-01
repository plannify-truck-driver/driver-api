FROM rust:1.92.0-slim-bookworm AS rust-build

WORKDIR /usr/local/src/user

RUN \
    apt-get update && \
    apt-get install -y --no-install-recommends \
    pkg-config \
    protobuf-compiler \
    libssl-dev && \
    rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY api/Cargo.toml ./api/
COPY core/Cargo.toml ./core/
COPY .sqlx ./sqlx

RUN \
    mkdir -p api/src core/src && \
    echo "fn main() {}" > api/src/main.rs && \
    touch core/src/lib.rs && \
    cargo build --release

COPY api api
COPY core core
COPY .sqlx .sqlx

RUN \
    touch api/src/main.rs && \
    touch core/src/lib.rs && \
    cargo build --release

FROM debian:bookworm-slim AS runtime

RUN \
    apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 && \
    rm -rf /var/lib/apt/lists/* && \
    addgroup \
    --system \
    --gid 1000 \
    plannify-user && \
    adduser \
    --system \
    --no-create-home \
    --disabled-login \
    --uid 1000 \
    --gid 1000 \
    plannify-user

USER plannify-user

FROM runtime AS api

COPY --from=rust-build /usr/local/src/user/target/release/api /usr/local/bin/
COPY --from=rust-build --chown=plannify-user:plannify-user /usr/local/src/user/core/templates /usr/local/src/user/core/templates

WORKDIR /usr/local/src/user

EXPOSE 3000

ENTRYPOINT ["api"]