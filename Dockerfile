# ── dev ──────────────────────────────────────────────────────────────────────
FROM rust:1.95-bookworm AS dev

ARG CARGO_LEPTOS_VERSION=0.2.47
RUN rustup target add wasm32-unknown-unknown \
    && cargo install cargo-leptos --version "${CARGO_LEPTOS_VERSION}" --locked \
    && cargo install sqlx-cli --locked

WORKDIR /app
EXPOSE 3000
CMD ["cargo", "leptos", "watch"]

# ── builder ───────────────────────────────────────────────────────────────────
FROM rust:1.95-bookworm AS builder

ARG CARGO_LEPTOS_VERSION=0.2.47
RUN rustup target add wasm32-unknown-unknown \
    && cargo install cargo-leptos --version "${CARGO_LEPTOS_VERSION}" --locked

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
