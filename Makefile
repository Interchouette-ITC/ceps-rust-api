# ceps-rust-api - developer targets

APP_NAME ?= ceps-rust-api
HUB_IMAGE ?= interchouette/ceps-rust-api
TAG ?= latest
APP_VERSION ?= $(shell awk '/^version = /{gsub(/"/, "", $$3); print $$3; exit}' Cargo.toml)
DOCKERFILE ?= docker/Dockerfile
DOCKER_BUILDKIT ?= 1
COMPOSE ?= docker/docker-compose.yml

FEATURES ?= ceps-all,swagger-ui,tx-return,sign-local,sign-kms,chain-put
CARGO_FEATURES := --features $(FEATURES)
RUST_LOG ?= info

CARGO := env -u CARGO_TARGET_DIR -u PLAYWRIGHT_BROWSERS_PATH cargo

CLIPPY_FLAGS := -D warnings -D clippy::all

.DEFAULT_GOAL := help

.PHONY: help build build-release check test verify verify-slices lint format format-check clippy \
	docker-build docker-run docker-run-kms docker-stop version-show run pem-ban \
	export-local-keys run-local fetch-wasm

NCTL_CONTAINER ?= casper-nctl-2-docker-dev
NCTL_USERS ?= 1 2 3
CEPS_CONTRACTS_TAG ?= dev-preview

help:
	@echo "ceps-rust-api targets"
	@echo "  make build / test / verify / verify-slices / run"
	@echo "  make fetch-wasm          # download tip CEP contracts into tests/wasm/"
	@echo "  make export-local-keys   # print LOCAL_KEYS_JSON (NCTL users; lab only)"
	@echo "  make run-local           # run with SIGN_BACKEND=local + exported keys"
	@echo "  Features: FEATURES=$(FEATURES)"
	@echo "  SIGN_BACKEND: none (default) | local (lab) | local-production | kms (recommended)"

build:
	$(CARGO) build -p ceps-rust-api $(CARGO_FEATURES)

build-release:
	$(CARGO) build -p ceps-rust-api --release $(CARGO_FEATURES)

check:
	$(CARGO) check -p ceps-rust-api $(CARGO_FEATURES)

format:
	$(CARGO) fmt --all

format-check:
	$(CARGO) fmt --all -- --check

clippy:
	$(CARGO) clippy -p ceps-rust-api $(CARGO_FEATURES) -- $(CLIPPY_FLAGS)

lint: format-check clippy pem-ban

test: lint
	$(CARGO) test -p ceps-rust-api $(CARGO_FEATURES) -- --nocapture

verify: lint test

# Same feature slices as CI (.github/workflows/ci.yml).
verify-slices:
	$(CARGO) test -p ceps-rust-api --no-default-features --features cep18,tx-return,swagger-ui -- --nocapture
	$(CARGO) test -p ceps-rust-api --no-default-features --features cep78,tx-return,swagger-ui -- --nocapture
	$(CARGO) check -p ceps-rust-api --no-default-features --features cep85,cep95,swagger-ui
	$(CARGO) test -p ceps-rust-api --no-default-features --features cep18,swagger-ui -- --nocapture
	$(CARGO) test -p ceps-rust-api --no-default-features --features swagger-ui -- --nocapture

pem-ban:
	@if rg -n 'secret_key\.pem|PATH_PRIVATE_KEY_|PRIVATE_KEY_|FAUCET_MODE|BOOTSTRAP_KMS' \
		crates docker README.md .env.example .github 2>/dev/null ; then \
		echo "pem-ban: forbidden key/PEM strings found"; exit 1; \
	fi
	@echo "pem-ban: ok"

run:
	RUST_LOG=$(RUST_LOG) $(CARGO) run -p ceps-rust-api $(CARGO_FEATURES)

# Demo tip CEP contracts (https://github.com/Interchouette-ITC/ceps-rust-ts-client releases).
fetch-wasm:
	@chmod +x scripts/fetch-ceps-contracts.sh
	@CEPS_CONTRACTS_TAG=$(CEPS_CONTRACTS_TAG) ./scripts/fetch-ceps-contracts.sh

# Lab only: dump NCTL user PEMs as LOCAL_KEYS_JSON (stdout). Never faucet.
export-local-keys:
	@chmod +x scripts/export-nctl-local-keys.sh
	@NCTL_CONTAINER=$(NCTL_CONTAINER) NCTL_USERS="$(NCTL_USERS)" ./scripts/export-nctl-local-keys.sh

# Lab only: run API with SIGN_BACKEND=local and keys from export-local-keys.
run-local:
	@chmod +x scripts/export-nctl-local-keys.sh
	@RUST_LOG=$(RUST_LOG) \
	SIGN_BACKEND=local \
	LOCAL_KEYS_JSON="$$(NCTL_CONTAINER=$(NCTL_CONTAINER) NCTL_USERS="$(NCTL_USERS)" ./scripts/export-nctl-local-keys.sh)" \
	$(CARGO) run -p ceps-rust-api $(CARGO_FEATURES)

docker-build:
	DOCKER_BUILDKIT=$(DOCKER_BUILDKIT) docker build -f $(DOCKERFILE) \
		--build-arg FEATURES=$(FEATURES) \
		-t $(HUB_IMAGE):$(TAG) -t $(HUB_IMAGE):$(APP_VERSION) \
		..

docker-run:
	docker compose -f $(COMPOSE) up -d

docker-run-kms:
	docker compose -f $(COMPOSE) --profile kms up -d

docker-stop:
	docker compose -f $(COMPOSE) --profile kms down || true
	docker compose -f $(COMPOSE) down

version-show:
	@echo "version=$(APP_VERSION) image=$(HUB_IMAGE):$(TAG)"
