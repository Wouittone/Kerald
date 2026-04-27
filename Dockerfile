# syntax=docker/dockerfile:1.23

ARG RUST_VERSION=1.95
ARG ALPINE_VERSION=3.23

# Build with the Rust musl Alpine toolchain required for lightweight Kerald
# runtime nodes.
FROM rust:${RUST_VERSION}-alpine AS builder

WORKDIR /workspace

RUN apk add --no-cache musl-dev

# Keep the build context focused on the broker/runtime Rust source. Future
# client sources and language bindings should not be copied into this image.
COPY Cargo.toml Cargo.lock rustfmt.toml ./
COPY core/src ./core/src

RUN cargo build --locked --release --bin kerald

# Runtime stage contains only the broker binary and minimal TLS roots.
FROM alpine:${ALPINE_VERSION} AS runtime

# Create a system group and disabled-login system user so the broker never runs
# as root. The home directory is the broker workdir for future local state.
RUN addgroup -S kerald \
    && adduser -S -D -G kerald -h /var/lib/kerald -s /sbin/nologin kerald \
    && apk add --no-cache ca-certificates

COPY --from=builder /workspace/target/release/kerald /usr/local/bin/kerald

USER kerald
WORKDIR /var/lib/kerald

# Mirrors the broker's current default QUIC inter-broker port.
EXPOSE 9000/udp

ENTRYPOINT ["/usr/local/bin/kerald"]
