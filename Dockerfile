FROM rust:1.99.1-slim-trixie AS rust-build

WORKDIR /usr/local/src/user

RUN \
    apt-get update && \
    apt-get install -y --no-install-recommends \
    pkg-config \
    protobuf-compiler \
    libssl-dev && \
    rm -rf /var/lib/apt/lists/*

RUN cargo install cargo-auditable --version 0.7.7 --locked

COPY Cargo.toml Cargo.lock ./
COPY api/Cargo.toml ./api/
COPY core/Cargo.toml ./core/
COPY job/Cargo.toml ./job/
COPY .sqlx ./sqlx

RUN \
    mkdir -p api/src core/src job/src && \
    echo "fn main() {}" > api/src/main.rs && \
    echo "fn main() {}" > job/src/main.rs && \
    touch core/src/lib.rs && \
    cargo auditable build --release

COPY api api
COPY core core
COPY job job
COPY .sqlx .sqlx

RUN \
    touch api/src/main.rs && \
    touch core/src/lib.rs && \
    touch job/src/main.rs && \
    cargo auditable build --release

# Distroless: glibc, libssl and CA certificates only, no shell or package manager
FROM gcr.io/distroless/cc-debian13 AS runtime

USER 1000:1000

FROM runtime AS api

COPY --from=rust-build /usr/local/src/user/target/release/api /usr/local/bin/
COPY --from=rust-build --chown=1000:1000 /usr/local/src/user/core/templates /usr/local/src/user/core/templates

WORKDIR /usr/local/src/user

EXPOSE 3000

ENTRYPOINT ["api"]

FROM runtime AS job

COPY --from=rust-build /usr/local/src/user/target/release/job /usr/local/bin/
COPY --from=rust-build --chown=1000:1000 /usr/local/src/user/core/templates /usr/local/src/user/core/templates

WORKDIR /usr/local/src/user

ENTRYPOINT ["job"]