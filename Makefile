# ceps-rust-api - developer targets

SHELL := /bin/bash
.SHELLFLAGS := -eu -o pipefail -c

ROOT := $(CURDIR)
APP_NAME ?= ceps-rust-api
HUB_IMAGE ?= interchouette/ceps-rust-api
GHCR_PERSONAL_IMAGE ?= ghcr.io/groussac/ceps-rust-api
GHCR_WORKER_IMAGE ?= ghcr.io/interchouette/ceps-rust-api
GHCR_ORG_IMAGE ?= ghcr.io/interchouette-itc/ceps-rust-api
TAG ?= latest
APP_VERSION ?= $(shell awk '/^version = /{gsub(/"/, "", $$3); print $$3; exit}' Cargo.toml)
MCP_NAME ?= ceps-rust-api-mcp
MCP_HUB_IMAGE ?= interchouette/ceps-rust-api-mcp
MCP_GHCR_PERSONAL_IMAGE ?= ghcr.io/groussac/ceps-rust-api-mcp
MCP_GHCR_WORKER_IMAGE ?= ghcr.io/interchouette/ceps-rust-api-mcp
MCP_GHCR_ORG_IMAGE ?= ghcr.io/interchouette-itc/ceps-rust-api-mcp
MCP_VERSION ?= $(shell awk '/^version = /{gsub(/"/, "", $$3); print $$3; exit}' mcp/Cargo.toml)
DOCKERFILE ?= docker/Dockerfile
DOCKER_BUILDKIT ?= 1
CI ?= 0
COMPOSE ?= docker/docker-compose.yml
COMPOSE_MCP ?= docker/docker-compose.mcp.yml

# Sibling tip CEP contract checkouts (override if needed).
CEP18_PRODUCT ?= $(ROOT)/../cep-18
CEP78_PRODUCT ?= $(ROOT)/../cep-78-enhanced-nft
CEP85_PRODUCT ?= $(ROOT)/../cep-1155
CEP95_PRODUCT ?= $(ROOT)/../cep-95
WASM_DIR := $(ROOT)/tests/wasm

FEATURES ?= ceps-all,swagger-ui,tx-return,sign-local,sign-kms,chain-put
CARGO_FEATURES := --features $(FEATURES)
RUST_LOG ?= info

CARGO := env -u CARGO_TARGET_DIR -u PLAYWRIGHT_BROWSERS_PATH cargo

CLIPPY_FLAGS := -D warnings -D clippy::all

.DEFAULT_GOAL := help

.PHONY: help build build-release check test verify verify-slices lint format format-check clippy \
	docker-build docker-build-dev docker-run docker-run-kms docker-stop version-show run pem-ban \
	export-local-keys run-local wasm-from-ceps \
	docker-push-dev-hub docker-push-dev-ghcr-personal docker-push-dev-ghcr-itc docker-push-dev \
	docker-push-release-hub docker-push-release-ghcr-personal docker-push-release-ghcr-itc \
	docker-push-release \
	mcp-build mcp-docker-build mcp-docker-build-dev \
	mcp-docker-push-dev-hub mcp-docker-push-dev-ghcr-personal mcp-docker-push-dev-ghcr-itc \
	mcp-docker-push-dev \
	mcp-docker-push-release-hub mcp-docker-push-release-ghcr-personal mcp-docker-push-release-ghcr-itc \
	mcp-docker-push-release mcp-http mcp-http-stop run-mcp run-mcp-http

NCTL_CONTAINER ?= casper-nctl-2-docker-dev
NCTL_USERS ?= 1 2 3

help:
	@echo "ceps-rust-api targets"
	@echo "  make build / test / verify / verify-slices / run"
	@echo "  make wasm-from-ceps      # stage tip CEP WASMs into tests/wasm/"
	@echo "  make export-local-keys   # print LOCAL_KEYS_JSON (NCTL users; lab only)"
	@echo "  make run-local           # run with SIGN_BACKEND=local + exported keys"
	@echo "  make docker-build / docker-build-dev / docker-push-dev"
	@echo "  make mcp-build / mcp-docker-build-dev / mcp-http / run-mcp"
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

# Stage tip contract WASMs from sibling CEP product checkouts into tests/wasm/
# (same as ceps-rust-ts-client). Commit the result when refreshing the pack.
wasm-from-ceps:
	@mkdir -p "$(WASM_DIR)"
	@set -euo pipefail; \
	for pair in \
		"$(CEP18_PRODUCT)|cep18" \
		"$(CEP78_PRODUCT)|cep78" \
		"$(CEP85_PRODUCT)|cep85" \
		"$(CEP95_PRODUCT)|cep95"; do \
		root="$${pair%%|*}"; name="$${pair##*|}"; \
		if [ ! -d "$$root" ]; then \
			echo "wasm-from-ceps: skip $$name (missing $$root)"; \
			continue; \
		fi; \
		tip="$$root/tests/wasm"; \
		if [ -d "$$tip" ] && find "$$tip" -type f -name '*.wasm' -print -quit | grep -q .; then \
			found=$$(find "$$tip" -type f -name '*.wasm' | sort); \
		else \
			found=$$( { \
				find "$$root" -type f -name '*.wasm' \
					! -path '*/target/debug/*' ! -path '*/node_modules/*' \
					! -path '*/.git/*' 2>/dev/null; \
			} | awk 'NF' | while read -r f; do \
				printf '%s\t%s\n' "$$(stat -c '%Y' "$$f" 2>/dev/null || echo 0)" "$$f"; \
			done | sort -nr | cut -f2- | awk -F/ '{ base=$$NF; if (!seen[base]++) print }'); \
		fi; \
		if [ -z "$$found" ]; then \
			echo "wasm-from-ceps: no wasm under $$root (build contracts there first)"; \
			continue; \
		fi; \
		rm -rf "$(WASM_DIR)/$$name"; \
		mkdir -p "$(WASM_DIR)/$$name"; \
		echo "$$found" | while read -r f; do \
			cp -f "$$f" "$(WASM_DIR)/$$name/"; \
			echo "  staged $$name/$$(basename "$$f")"; \
		done; \
	done; \
	echo "wasm-from-ceps: done -> $(WASM_DIR)"

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

docker-build-dev:
	DOCKER_BUILDKIT=$(DOCKER_BUILDKIT) docker build -f $(DOCKERFILE) \
		--build-arg FEATURES=$(FEATURES) \
		-t $(HUB_IMAGE):dev \
		-t $(GHCR_PERSONAL_IMAGE):dev \
		-t $(GHCR_WORKER_IMAGE):dev \
		-t $(GHCR_ORG_IMAGE):dev \
		..

docker-push-dev-hub:
	docker push $(HUB_IMAGE):dev

docker-push-dev-ghcr-personal:
	docker push $(GHCR_PERSONAL_IMAGE):dev

docker-push-dev-ghcr-itc:
	docker push $(GHCR_WORKER_IMAGE):dev
	docker push $(GHCR_ORG_IMAGE):dev

docker-push-dev:
	@if [ "$(CI)" = "1" ]; then \
		echo "Use docker-push-dev-hub / docker-push-dev-ghcr-* in CI"; \
		exit 1; \
	fi
	@echo "Logging in to Docker Hub..."; \
	docker login || { echo "Docker Hub login failed"; exit 1; }
	$(MAKE) docker-push-dev-hub
	@echo "Logging in to GHCR (personal)..."; \
	docker login ghcr.io || { echo "Skipping personal GHCR"; exit 0; }
	$(MAKE) docker-push-dev-ghcr-personal
	@echo "Logging in to GHCR (org)..."; \
	docker login ghcr.io || { echo "Skipping org GHCR"; exit 0; }
	$(MAKE) docker-push-dev-ghcr-itc

docker-push-release-hub:
	docker push $(HUB_IMAGE):$(APP_VERSION)
	docker push $(HUB_IMAGE):latest

docker-push-release-ghcr-personal:
	docker tag $(HUB_IMAGE):$(APP_VERSION) $(GHCR_PERSONAL_IMAGE):$(APP_VERSION)
	docker tag $(HUB_IMAGE):latest $(GHCR_PERSONAL_IMAGE):latest
	docker push $(GHCR_PERSONAL_IMAGE):$(APP_VERSION)
	docker push $(GHCR_PERSONAL_IMAGE):latest

docker-push-release-ghcr-itc:
	docker tag $(HUB_IMAGE):$(APP_VERSION) $(GHCR_WORKER_IMAGE):$(APP_VERSION)
	docker tag $(HUB_IMAGE):latest $(GHCR_WORKER_IMAGE):latest
	docker tag $(HUB_IMAGE):$(APP_VERSION) $(GHCR_ORG_IMAGE):$(APP_VERSION)
	docker tag $(HUB_IMAGE):latest $(GHCR_ORG_IMAGE):latest
	docker push $(GHCR_WORKER_IMAGE):$(APP_VERSION)
	docker push $(GHCR_WORKER_IMAGE):latest
	docker push $(GHCR_ORG_IMAGE):$(APP_VERSION)
	docker push $(GHCR_ORG_IMAGE):latest

docker-push-release: docker-push-release-hub \
	docker-push-release-ghcr-personal docker-push-release-ghcr-itc

docker-run:
	docker compose -f $(COMPOSE) up -d

docker-run-kms:
	docker compose -f $(COMPOSE) --profile kms up -d

docker-stop:
	docker compose -f $(COMPOSE) --profile kms down || true
	docker compose -f $(COMPOSE) down

version-show:
	@echo "version=$(APP_VERSION) image=$(HUB_IMAGE):$(TAG)"
	@echo "mcp_version=$(MCP_VERSION) mcp_image=$(MCP_HUB_IMAGE):$(TAG)"

# ---------------------------------------------------------------------------
# MCP sidecar (mcp/ - separate Cargo package; no dep on API lib)
# ---------------------------------------------------------------------------

mcp-build:
	$(CARGO) build --manifest-path mcp/Cargo.toml --release

mcp-docker-build:
	DOCKER_BUILDKIT=$(DOCKER_BUILDKIT) docker build --network=host \
		-t $(MCP_NAME):$(TAG) \
		-t $(MCP_HUB_IMAGE):$(TAG) \
		-t $(MCP_HUB_IMAGE):$(MCP_VERSION) \
		-f mcp/Dockerfile \
		mcp

mcp-docker-build-dev:
	DOCKER_BUILDKIT=$(DOCKER_BUILDKIT) docker build --network=host \
		-t $(MCP_NAME):dev \
		-t $(MCP_HUB_IMAGE):dev \
		-t $(MCP_GHCR_PERSONAL_IMAGE):dev \
		-t $(MCP_GHCR_WORKER_IMAGE):dev \
		-t $(MCP_GHCR_ORG_IMAGE):dev \
		-f mcp/Dockerfile \
		mcp

mcp-docker-push-dev-hub:
	docker push $(MCP_HUB_IMAGE):dev

mcp-docker-push-dev-ghcr-personal:
	docker push $(MCP_GHCR_PERSONAL_IMAGE):dev

mcp-docker-push-dev-ghcr-itc:
	docker push $(MCP_GHCR_WORKER_IMAGE):dev
	docker push $(MCP_GHCR_ORG_IMAGE):dev

mcp-docker-push-dev:
	@if [ "$(CI)" = "1" ]; then \
		echo "Use mcp-docker-push-dev-hub / mcp-docker-push-dev-ghcr-* in CI"; \
		exit 1; \
	fi
	@echo "Logging in to Docker Hub..."; \
	docker login || { echo "Docker Hub login failed"; exit 1; }
	$(MAKE) mcp-docker-push-dev-hub
	@echo "Logging in to GHCR (personal)..."; \
	docker login ghcr.io || { echo "Skipping personal GHCR"; exit 0; }
	$(MAKE) mcp-docker-push-dev-ghcr-personal
	@echo "Logging in to GHCR (org)..."; \
	docker login ghcr.io || { echo "Skipping org GHCR"; exit 0; }
	$(MAKE) mcp-docker-push-dev-ghcr-itc

mcp-docker-push-release-hub:
	docker push $(MCP_HUB_IMAGE):$(MCP_VERSION)
	docker push $(MCP_HUB_IMAGE):latest

mcp-docker-push-release-ghcr-personal:
	docker tag $(MCP_HUB_IMAGE):$(MCP_VERSION) $(MCP_GHCR_PERSONAL_IMAGE):$(MCP_VERSION)
	docker tag $(MCP_HUB_IMAGE):latest $(MCP_GHCR_PERSONAL_IMAGE):latest
	docker push $(MCP_GHCR_PERSONAL_IMAGE):$(MCP_VERSION)
	docker push $(MCP_GHCR_PERSONAL_IMAGE):latest

mcp-docker-push-release-ghcr-itc:
	docker tag $(MCP_HUB_IMAGE):$(MCP_VERSION) $(MCP_GHCR_WORKER_IMAGE):$(MCP_VERSION)
	docker tag $(MCP_HUB_IMAGE):latest $(MCP_GHCR_WORKER_IMAGE):latest
	docker tag $(MCP_HUB_IMAGE):$(MCP_VERSION) $(MCP_GHCR_ORG_IMAGE):$(MCP_VERSION)
	docker tag $(MCP_HUB_IMAGE):latest $(MCP_GHCR_ORG_IMAGE):latest
	docker push $(MCP_GHCR_WORKER_IMAGE):$(MCP_VERSION)
	docker push $(MCP_GHCR_WORKER_IMAGE):latest
	docker push $(MCP_GHCR_ORG_IMAGE):$(MCP_VERSION)
	docker push $(MCP_GHCR_ORG_IMAGE):latest

mcp-docker-push-release: mcp-docker-push-release-hub \
	mcp-docker-push-release-ghcr-personal mcp-docker-push-release-ghcr-itc

mcp-http:
	-docker pull $(MCP_HUB_IMAGE):$(MCP_VERSION)
	@if ! docker image inspect $(MCP_HUB_IMAGE):$(MCP_VERSION) >/dev/null 2>&1 \
		&& ! docker image inspect $(MCP_NAME):$(MCP_VERSION) >/dev/null 2>&1; then \
		echo "Hub image missing; building locally…"; \
		$(MAKE) mcp-docker-build; \
	fi
	CEPS_API_MCP_IMAGE=$(MCP_HUB_IMAGE):$(MCP_VERSION) CEPS_HOST_ROOT="$(CURDIR)" \
		docker compose -f $(COMPOSE_MCP) up -d --force-recreate

mcp-http-stop:
	-CEPS_HOST_ROOT="$(CURDIR)" docker compose -f $(COMPOSE_MCP) down --remove-orphans
	-docker stop ceps-rust-api-mcp 2>/dev/null
	-docker rm ceps-rust-api-mcp 2>/dev/null

run-mcp:
	CEPS_API_ROOT="$(CURDIR)" CEPS_HOST_ROOT="$(CURDIR)" \
		$(CARGO) run --manifest-path mcp/Cargo.toml --quiet --

run-mcp-http:
	CEPS_API_ROOT="$(CURDIR)" CEPS_HOST_ROOT="$(CURDIR)" \
		$(CARGO) run --manifest-path mcp/Cargo.toml --quiet -- \
		--http --listen 127.0.0.1:4790
