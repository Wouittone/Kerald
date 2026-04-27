# syntax=docker/dockerfile:1.7

ARG RUST_VERSION=1.95
ARG ALPINE_VERSION=3.22

FROM rust:${RUST_VERSION}-alpine AS builder

WORKDIR /workspace

RUN apk add --no-cache musl-dev

COPY Cargo.toml Cargo.lock rustfmt.toml ./
COPY core/src ./core/src

RUN cargo build --locked --release --bin kerald

FROM alpine:${ALPINE_VERSION} AS runtime

RUN addgroup -S kerald \
    && adduser -S -D -G kerald -h /var/lib/kerald -s /sbin/nologin kerald \
    && apk add --no-cache ca-certificates

COPY --from=builder /workspace/target/release/kerald /usr/local/bin/kerald

USER kerald
WORKDIR /var/lib/kerald

# Mirrors the broker's current default QUIC inter-broker port.
EXPOSE 9000/udp

ENTRYPOINT ["/usr/local/bin/kerald"]
