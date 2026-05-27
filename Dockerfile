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

# ── extract runtime libs beyond what glibc-dynamic provides ──────────────────
# ldd finds all deps; we filter out glibc/libgcc (already in chainguard base)
# and copy the rest (currently just liblzma) in an arch-agnostic way.
FROM builder AS runtime-libs
RUN mkdir -p /extra-libs \
    && ldd /app/target/release/api-mock-server \
       | grep "=> /" \
       | awk '{print $3}' \
       | grep -Ev '/(libc|libm|libgcc_s|libpthread|libdl|librt)\.so' \
       | xargs -I{} sh -c 'cp -L {} /extra-libs/$(basename {})'

# ── pre-create data dir owned by chainguard nonroot uid 65532 ─────────────────
FROM builder AS data-dir
RUN mkdir -p /app/data && touch /app/data/.keep

# ── prod ──────────────────────────────────────────────────────────────────────
# chainguard/glibc-dynamic: distroless, no shell, glibc + libgcc + ca-certs
FROM cgr.dev/chainguard/glibc-dynamic:latest AS prod

# Extra runtime libraries the binary needs (liblzma)
COPY --from=runtime-libs /extra-libs/ /usr/lib/

WORKDIR /app

COPY --from=builder --chown=65532:65532 /app/target/release/api-mock-server ./server
COPY --from=builder --chown=65532:65532 /app/site ./site
COPY --from=data-dir --chown=65532:65532 /app/data ./data

ENV LEPTOS_SITE_ROOT=/app/site
ENV DATABASE_URL=sqlite:///app/data/app.db
VOLUME ["/app/data"]
EXPOSE 3000
CMD ["./server"]
