# Container Images

Kerald production containers must stay lightweight and operationally simple.
Images are built with a musl Alpine multi-stage flow:

- The builder stage uses the Rust 1.95 Alpine toolchain and compiles the
  `kerald` broker binary with `cargo build --locked --release --bin kerald`.
- The runtime stage uses Alpine and copies only the compiled broker binary plus
  minimal certificate support.
- The runtime process runs as an unprivileged `kerald` user with no shell.
- Production runs should drop all Linux capabilities and set Docker's
  `no-new-privileges` security option.
- The default exposed inter-broker port is `9000/udp`, matching the current QUIC
  protocol baseline and the broker's current default development port.

Build the image from the repository root:

```sh
docker build -t kerald:local .
```

Run a production-ready single-node broker with the built image:

```sh
docker run --rm \
  --cap-drop=ALL \
  --security-opt no-new-privileges \
  -p 9000:9000/udp \
  kerald:local
```

For local development only, the security flags may be omitted:

```sh
docker run --rm -p 9000:9000/udp kerald:local
```

Run with a mounted broker configuration:

```sh
docker run --rm \
  --cap-drop=ALL \
  --security-opt no-new-privileges \
  -p 9000:9000/udp \
  -v "$PWD/kerald.toml:/etc/kerald/kerald.toml:ro" \
  kerald:local --config /etc/kerald/kerald.toml
```

Container changes must continue to preserve partitionless topic semantics,
safety-first write admission, timestamp cursor semantics, and the broker
boundary that keeps Lance persistence and OpenDAL storage responsibilities out
of the runtime image unless those dependencies are required by the broker
binary.
