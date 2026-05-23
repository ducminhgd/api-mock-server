.PHONY: dev build test migrate lint

dev:
	cargo leptos watch

build:
	cargo leptos build --release

test:
	cargo test --features ssr

migrate:
	sqlx migrate run

lint:
	cargo clippy --fix --allow-dirty
	cargo fmt --all

pre-commit: lint test build