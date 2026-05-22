# ── dev stage ─────────────────────────────────────────────────────────────────
FROM cgr.dev/chainguard/go:latest-dev AS dev

USER root
RUN apk add --no-cache build-base && \
    go install github.com/air-verse/air@latest

WORKDIR /app

COPY go.mod go.sum ./
RUN go mod download

EXPOSE 8080

CMD ["air"]

# ── builder stage ─────────────────────────────────────────────────────────────
FROM cgr.dev/chainguard/go:latest-dev AS builder

USER root
RUN apk add --no-cache build-base

WORKDIR /app

COPY go.mod go.sum ./
RUN go mod download

COPY . .
RUN CGO_ENABLED=1 go build -ldflags="-s -w" -o bin/server ./cmd/server

# ── prod stage ────────────────────────────────────────────────────────────────
# glibc-dynamic is required because the SQLite driver links against glibc via CGO.
FROM cgr.dev/chainguard/glibc-dynamic AS prod

WORKDIR /app

COPY --from=builder /app/bin/server ./server

EXPOSE 8080

ENTRYPOINT ["/app/server"]
