# ── dev ──────────────────────────────────────────────────────────────────────
FROM rust:1.87-bookworm AS dev

RUN rustup target add wasm32-unknown-unknown \
    && cargo install cargo-leptos sqlx-cli --locked

WORKDIR /app
EXPOSE 3000
CMD ["cargo", "leptos", "watch"]

# ── builder ───────────────────────────────────────────────────────────────────
FROM cgr.dev/chainguard/rust:latest-dev AS builder

RUN rustup target add wasm32-unknown-unknown \
    && cargo install cargo-leptos --locked

WORKDIR /app
COPY . .

RUN cargo leptos build --release

# ── prod ──────────────────────────────────────────────────────────────────────
FROM cgr.dev/chainguard/glibc-dynamic AS prod

WORKDIR /app
COPY --from=builder --chown=nonroot:nonroot /app/target/release/api-mock-server ./server
COPY --from=builder --chown=nonroot:nonroot /app/target/site ./site

ENV LEPTOS_SITE_ROOT=/app/site
EXPOSE 3000
CMD ["./server"]
