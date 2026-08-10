# syntax=docker/dockerfile:1
# Build from the parent of this repo (sibling checkouts of ceps-rust-ts-client + rustSDK):
#   docker build -f ceps-rust-api/docker/Dockerfile -t interchouette/ceps-rust-api:dev .
# Or from this repo: make docker-build (uses .. as context).

FROM rust:slim-trixie AS builder

ARG FEATURES=ceps-all,swagger-ui,tx-return,sign-local,sign-kms,chain-put

RUN apt-get update && apt-get install -y --no-install-recommends \
  pkg-config \
  libssl-dev \
  ca-certificates \
  build-essential \
  curl \
  && rm -rf /var/lib/apt/lists/*

WORKDIR /workspace

COPY ceps-rust-api/Cargo.toml ceps-rust-api/Cargo.lock ceps-rust-api/LICENSE ceps-rust-api/README.md ./ceps-rust-api/
COPY ceps-rust-api/crates/ceps-api/Cargo.toml ./ceps-rust-api/crates/ceps-api/Cargo.toml
COPY ceps-rust-ts-client ./ceps-rust-ts-client
COPY rustSDK ./rustSDK

WORKDIR /workspace/ceps-rust-api

RUN mkdir -p crates/ceps-api/src \
  && echo 'fn main() {}' > crates/ceps-api/src/main.rs \
  && echo '' > crates/ceps-api/src/lib.rs

RUN cargo build --release -p ceps-rust-api --features "${FEATURES}" || true

RUN rm -rf crates/ceps-api/src
COPY ceps-rust-api/crates/ceps-api/src crates/ceps-api/src

RUN touch crates/ceps-api/src/main.rs crates/ceps-api/src/lib.rs \
  && cargo build --release -p ceps-rust-api --features "${FEATURES}"

FROM gcr.io/distroless/cc-debian13:nonroot AS runtime

WORKDIR /app
COPY --from=builder /workspace/ceps-rust-api/target/release/ceps-rust-api /app/ceps-rust-api

ENV APP_ADDR=0.0.0.0
ENV APP_PORT=8080
EXPOSE 8080

USER nonroot:nonroot
ENTRYPOINT ["/app/ceps-rust-api"]
