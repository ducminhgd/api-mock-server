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
	cargo clippy --all-targets --all-features -- -D warnings
	cargo fmt --check
