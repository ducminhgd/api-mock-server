# ── dev ──────────────────────────────────────────────────────────────────────
FROM rust:1.95-bookworm AS dev

ARG CARGO_LEPTOS_VERSION=0.3.6
RUN rustup target add wasm32-unknown-unknown \
    && cargo install cargo-leptos --version "${CARGO_LEPTOS_VERSION}" --locked \
    && cargo install sqlx-cli --no-default-features --features sqlite --locked

WORKDIR /app
EXPOSE 3000
CMD ["cargo", "leptos", "watch"]

# ── builder ───────────────────────────────────────────────────────────────────
FROM rust:1.95-bookworm AS builder

ARG CARGO_LEPTOS_VERSION=0.3.6
RUN rustup target add wasm32-unknown-unknown \
    && cargo install cargo-leptos --version "${CARGO_LEPTOS_VERSION}" --locked

WORKDIR /app
COPY . .

RUN cargo leptos build --release

# ── prod ──────────────────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS prod

RUN apt-get update && apt-get install -y --no-install-recommends \
        libssl3 ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m -u 1000 appuser
WORKDIR /app

COPY --from=builder --chown=appuser:appuser /app/target/release/api-mock-server ./server
COPY --from=builder --chown=appuser:appuser /app/target/site ./site

RUN mkdir -p /app/data && chown appuser:appuser /app/data

USER appuser

ENV LEPTOS_SITE_ROOT=/app/site
ENV DATABASE_URL=sqlite:///app/data/app.db
VOLUME ["/app/data"]
EXPOSE 3000
CMD ["./server"]
