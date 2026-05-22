# ── dev stage ─────────────────────────────────────────────────────────────────
FROM golang:1.24-alpine AS dev

RUN apk add --no-cache git && \
    go install github.com/air-verse/air@latest

WORKDIR /app

COPY go.mod go.sum ./
RUN go mod download

EXPOSE 8080

CMD ["air"]

# ── builder stage ─────────────────────────────────────────────────────────────
FROM golang:1.24-alpine AS builder

WORKDIR /app

COPY go.mod go.sum ./
RUN go mod download

COPY . .
RUN CGO_ENABLED=1 GOOS=linux go build -ldflags="-s -w" -o bin/server ./cmd/server

# ── prod stage ────────────────────────────────────────────────────────────────
FROM alpine:3.21 AS prod

RUN apk add --no-cache ca-certificates tzdata

WORKDIR /app

COPY --from=builder /app/bin/server ./server

EXPOSE 8080

CMD ["./server"]
