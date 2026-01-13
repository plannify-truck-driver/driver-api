FROM 1.92.0-slim-bookworm AS rust-build

WORKDIR /usr/local/src/user

COPY Cargo.toml Cargo.lock ./
COPY api/Cargo.toml ./api/
COPY core/Cargo.toml ./core/

RUN \
    mkdir -p api/src core/src && \
    echo "fn main() {}" > api/src/main.rs && \
    touch core/src/lib.rs && \
    cargo build --release

COPY migrations migrations
COPY api api
COPY core core

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
COPY --from=rust-build /usr/local/src/user/migrations /usr/local/src/user/migrations

WORKDIR /usr/local/src/user

EXPOSE 3000

ENTRYPOINT ["api"]
CMD ["run"]