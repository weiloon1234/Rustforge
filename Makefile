SHELL := /bin/bash

.PHONY: help
help:
	@echo "Rustforge Framework Makefile"
	@echo "--------------------------"
	@echo "  make check             cargo check --workspace"
	@echo "  make test              cargo test --workspace"
	@echo "  make fmt               cargo fmt --all"
	@echo "  make clippy            cargo clippy --workspace --all-targets --all-features -- -D warnings"
	@echo "  make docs-build        build framework docs frontend"
	@echo "  make scaffold-smoke    generate starter scaffold into /tmp/rustforge-starter"
	@echo "  make clean             cargo clean"

.PHONY: check
check:
	cargo check --workspace

.PHONY: test
test:
	cargo test --workspace

.PHONY: fmt
fmt:
	cargo fmt --all

.PHONY: clippy
clippy:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

.PHONY: docs-build
docs-build:
	npm --prefix core-docs/frontend run build

.PHONY: scaffold-smoke
scaffold-smoke:
	cargo run --manifest-path scaffold/Cargo.toml -- --output /tmp/rustforge-starter --force

.PHONY: clean
clean:
	cargo clean
