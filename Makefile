# SPDX-FileCopyrightText: Copyright Logically Magic Inc. <hello@logicallymagic.com>
# SPDX-FileCopyrightText: Copyright Hamid Jahanzad <Hamid@Jahanzad.com>
# SPDX-FileCopyrightText: Copyright Contributors to the Bank8:SDK
# SPDX-License-Identifier: Apache-2.0

# Makefile — bank8-sdk (lib repo, RS §4.5)
#
# Self-contained dev controls for THIS lib. Sources the root .env for
# fleet env, guarantees the dev infra, and uses an ABSOLUTE
# SENTIN8_CAPTAIN_BINARY_PREFIX (cwd-independent — RS §4.5.1-1).
SHELL := /bin/bash
CARGO ?= cargo
GROUPNAME := bank8
RELEASE ?=

.PHONY: help build test lint tag clean

help: ## Show this help
	@echo "bank8-sdk (SDK/lib)"
	@grep -E '^[a-z-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-12s\033[0m %s\n", $$1, $$2}'

build: ## Build the workspace (dev profile; RELEASE=--release for release)
	@$(CARGO) build --workspace $(RELEASE)

test: ## Run every workspace test suite
	@$(CARGO) test --workspace $(RELEASE)

lint: ## cargo fmt --all -- --check
	@$(CARGO) fmt --all -- --check

tag: ## Print this SDK's pin tag (RS §7.1: sdk-vX / bank8-vX)
	@v=$$(awk -F'"' '/^version = /{print $$2; exit}' Cargo.toml); \
	if [ "$(GROUPNAME)" = "sentin8" ]; then echo "sdk-v$$v"; else echo "$(GROUPNAME)-v$$v"; fi

clean: ## cargo clean
	@$(CARGO) clean
	@rm -rf .run .run/logs .tmp
