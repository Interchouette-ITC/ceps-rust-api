# ceps-rust-api - developer targets

APP_NAME ?= ceps-rust-api
HUB_IMAGE ?= interchouette/ceps-rust-api
TAG ?= latest
APP_VERSION ?= $(shell awk '/^version = /{gsub(/"/, "", $$3); print $$3; exit}' Cargo.toml)
DOCKERFILE ?= docker/Dockerfile
DOCKER_BUILDKIT ?= 1
COMPOSE ?= docker/docker-compose.yml

# cep18 | cep78 | cep85 | cep95 | all
FEATURES ?= all
CARGO_FEATURES := --features $(FEATURES),swagger-ui

# Never compile into Cursor sandbox cache
CARGO := env -u CARGO_TARGET_DIR -u PLAYWRIGHT_BROWSERS_PATH cargo

CLIPPY_FLAGS := -D warnings -D clippy::all -D clippy::pedantic

.DEFAULT_GOAL := help

.PHONY: help build build-release check test verify lint format format-check clippy \
	docker-build docker-run docker-run-kms docker-stop version-show run pem-ban

help:
	@echo "ceps-rust-api targets"
	@echo ""
	@echo "  make build / build-release / check / test / lint / verify"
	@echo "  make run                 cargo run API (port APP_PORT or 8080)"
	@echo "  make docker-build / docker-run / docker-run-kms / docker-stop"
	@echo "  make pem-ban             fail if PEM/faucet key strings appear"
	@echo "  make version-show"
	@echo ""
	@echo "Features: FEATURES=$(FEATURES) (cep18|cep78|cep85|cep95|all)"
	@echo "SIGN_BACKEND: leave unset for none (no signer)"

build:
	$(CARGO) build -p ceps-api $(CARGO_FEATURES)

build-release:
	$(CARGO) build -p ceps-api --release $(CARGO_FEATURES)

check:
	$(CARGO) check -p ceps-api --locked $(CARGO_FEATURES)

format:
	$(CARGO) fmt --all

format-check:
	$(CARGO) fmt --all -- --check

clippy:
	$(CARGO) clippy -p ceps-api $(CARGO_FEATURES) -- $(CLIPPY_FLAGS)

lint: format-check clippy pem-ban

test: lint
	$(CARGO) test -p ceps-api $(CARGO_FEATURES) -- --nocapture

verify: lint test

pem-ban:
	@if rg -n 'secret_key\.pem|PATH_PRIVATE_KEY_|PRIVATE_KEY_|FAUCET_MODE|BOOTSTRAP_KMS' \
		crates docker README.md .env.example .github 2>/dev/null ; then \
		echo "pem-ban: forbidden key/PEM strings found"; exit 1; \
	fi
	@echo "pem-ban: ok"

run:
	$(CARGO) run -p ceps-api $(CARGO_FEATURES)

docker-build:
	DOCKER_BUILDKIT=$(DOCKER_BUILDKIT) docker build -f $(DOCKERFILE) -t $(HUB_IMAGE):$(TAG) -t $(HUB_IMAGE):$(APP_VERSION) .

docker-run:
	docker compose -f $(COMPOSE) up -d

docker-run-kms:
	docker compose -f $(COMPOSE) --profile kms up -d

docker-stop:
	docker compose -f $(COMPOSE) --profile kms down
	docker compose -f $(COMPOSE) down

version-show:
	@echo "version=$(APP_VERSION) image=$(HUB_IMAGE):$(TAG)"
