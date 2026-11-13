# syntax=docker/dockerfile:1

# ---- build ----
FROM rust:1-bookworm AS builder
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
RUN cargo build --release -p blazers --locked

# ---- runtime ----
FROM debian:bookworm-slim
# ca-certificates: required for outbound HTTPS (webhooks, OAuth, SMTP).
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --system blazers \
    && useradd --system --gid blazers --home-dir /data \
        --shell /usr/sbin/nologin blazers \
    && mkdir -p /data \
    && chown blazers:blazers /data

COPY --from=builder /src/target/release/blazers /usr/local/bin/blazers

ENV BLAZERS_DATA=/data \
    PORT=8080
VOLUME ["/data"]
EXPOSE 8080

USER blazers
CMD ["blazers", "serve"]
