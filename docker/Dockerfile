# syntax=docker/dockerfile:1
FROM rust:slim-trixie AS builder

# Match Makefile FEATURES (demo image: all CEP + add-ons). Override with --build-arg.
ARG FEATURES=ceps-all,swagger-ui,tx-return,sign-local,sign-kms,chain-put

RUN apt-get update && apt-get install -y --no-install-recommends \
  pkg-config \
  libssl-dev \
  ca-certificates \
  build-essential \
  && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY crates/ceps-api/Cargo.toml crates/ceps-api/Cargo.toml
COPY LICENSE README.md ./

RUN mkdir -p crates/ceps-api/src \
  && echo 'fn main() {}' > crates/ceps-api/src/main.rs \
  && echo '' > crates/ceps-api/src/lib.rs

RUN cargo build --release -p ceps-api --features "${FEATURES}" || true

RUN rm -rf crates/ceps-api/src
COPY crates/ceps-api/src crates/ceps-api/src

RUN touch crates/ceps-api/src/main.rs crates/ceps-api/src/lib.rs \
  && cargo build --release -p ceps-api --features "${FEATURES}"

FROM gcr.io/distroless/cc-debian13:nonroot AS runtime

WORKDIR /app
COPY --from=builder /app/Cargo.toml /app/Cargo.toml
COPY --from=builder /app/target/release/ceps-rust-api /app/ceps-rust-api

ENV APP_ADDR=0.0.0.0
ENV APP_PORT=8080
EXPOSE 8080

USER nonroot:nonroot
ENTRYPOINT ["/app/ceps-rust-api"]
