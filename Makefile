.PHONY: install dev build test migrate migrate-revert lint pre-commit setup check create-admin

# Load .env if it exists
ifneq (,$(wildcard .env))
  include .env
  export
endif

DATABASE_URL ?= sqlite://./dev.db
JWT_SECRET   ?= dev-secret

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
	cargo check --features ssr
	cargo check --features hydrate --target wasm32-unknown-unknown

pre-commit: lint test build

# First-time setup: install toolchain deps and run migrations
setup:
	rustup target add wasm32-unknown-unknown
	cargo install cargo-leptos --version 0.3.6 --locked
	cargo install sqlx-cli --no-default-features --features sqlite --locked
	DATABASE_URL=$(DATABASE_URL) sqlx database create
	DATABASE_URL=$(DATABASE_URL) sqlx migrate run