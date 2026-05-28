.PHONY: clear-cache install dev build test migrate migrate-revert lint pre-commit setup check create-admin

# Load .env if it exists
ifneq (,$(wildcard .env))
  include .env
  export
endif

HOST_TARGET := $(shell rustc -vV | grep host | awk '{print $$2}')
WASM_TARGET := wasm32-unknown-unknown
DATABASE_URL ?= sqlite://./dev.db
JWT_SECRET   ?= dev-secret

clear-cache:
	cargo clean
	rm -rf $(HOME)/.cargo/registry
	rm -rf $(HOME)/.cargo/git
	rm -rf Cargo.lock
	
install:
	cargo update
	cargo install cargo-leptos --version 0.3.6 --locked
	cargo install sqlx-cli --no-default-features --features sqlite --locked
	cargo install cargo-watch --locked

dev:
	DATABASE_URL=$(DATABASE_URL) JWT_SECRET=$(JWT_SECRET) cargo leptos watch

build:
	cargo leptos build --release

test:
	DATABASE_URL=$(DATABASE_URL) cargo test --features ssr

migrate:
	DATABASE_URL=$(DATABASE_URL) sqlx database create
	DATABASE_URL=$(DATABASE_URL) sqlx migrate run

# Usage: make create-admin ADMIN_USER=myuser  (password is prompted securely)
create-admin:
	@test -n "$(ADMIN_USER)" || (echo "Error: ADMIN_USER is required. Usage: make create-admin ADMIN_USER=<user>" && exit 1)
	cargo build --features ssr --bin create-admin -q
	DATABASE_URL=$(DATABASE_URL) ./target/debug/create-admin "$(ADMIN_USER)"

migrate-revert:
	DATABASE_URL=$(DATABASE_URL) sqlx migrate revert

lint:
	cargo clippy --fix --allow-dirty
	cargo fmt --all

check:
	rustup target add $(WASM_TARGET)
	cargo check --features ssr
	cargo check --features hydrate --target $(WASM_TARGET)

pre-commit: lint test build

# First-time setup: install toolchain deps and run migrations
setup: clear-cache
	rustup target add $(WASM_TARGET)
	cargo install cargo-leptos --version 0.3.6 --locked
	cargo install sqlx-cli --no-default-features --features sqlite --locked
	DATABASE_URL=$(DATABASE_URL) sqlx database create
	DATABASE_URL=$(DATABASE_URL) sqlx migrate run