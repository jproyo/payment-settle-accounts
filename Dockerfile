# syntax=docker/dockerfile:1.3-labs
FROM rust:buster AS builder
WORKDIR /app
RUN apt update -y
RUN apt install -y cmake

RUN cargo new --lib /app/payment
COPY Cargo.toml /app/payment
COPY Cargo.lock /app/payment
RUN --mount=type=cache,target=/usr/local/cargo/registry cd /app/payment && cargo build --release

COPY . /app/payment
RUN --mount=type=cache,target=/usr/local/cargo/registry <<EOF
set -e
touch /app/payment/src/lib.rs
cd /app/payment
cargo build --release
EOF

RUN mv /app/payment/target/release/payment-settle-accounts /app

# We do not need the Rust toolchain to run the binary!
FROM debian:buster-slim AS runtime
WORKDIR /app
RUN apt update -y
RUN apt install -y libpq5 ca-certificates
COPY --from=builder /app/payment-settle-accounts /app
ENTRYPOINT ["/app/payment-settle-accounts"]
