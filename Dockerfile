# ── frontend builder ──────────────────────────────────────────────────────────
FROM cgr.dev/chainguard/node:latest-dev AS frontend-builder

WORKDIR /app

COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci

COPY frontend/ .
RUN npm run build

# ── backend builder ───────────────────────────────────────────────────────────
FROM cgr.dev/chainguard/go:latest-dev AS backend-builder

USER root
RUN apk add --no-cache build-base

WORKDIR /app

COPY backend/go.mod backend/go.sum ./
RUN go mod download

COPY backend/ .
RUN CGO_ENABLED=1 go build -ldflags="-s -w" -o bin/server ./cmd/server

# ── prod stage ────────────────────────────────────────────────────────────────
# Pin to a digest before shipping to production:
#   docker pull debian:12-slim
#   docker inspect debian:12-slim --format='{{index .RepoDigests 0}}'
# Then replace "debian:12-slim" with "debian@sha256:<digest>".
#
# debian:12-slim is chosen because the Go binary links against glibc via CGO
# (SQLite driver). Chainguard's distroless images also provide glibc but have
# no shell or process-management tooling needed here.
FROM debian:12-slim AS prod

# nginx      — web server and reverse proxy
# libcap2-bin — provides setcap so nginx can bind port 80 as a non-root user
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        nginx \
        libcap2-bin \
    && setcap 'cap_net_bind_service=+ep' /usr/sbin/nginx \
    && rm -rf /var/lib/apt/lists/*

RUN adduser --system --no-create-home --uid 1001 appuser

COPY --from=frontend-builder /app/dist   /usr/share/nginx/html
COPY --from=backend-builder  /app/bin/server /app/server
COPY docker/nginx.conf   /etc/nginx/nginx.conf
COPY docker/entrypoint.sh /entrypoint.sh

RUN chmod +x /entrypoint.sh /app/server

USER appuser

EXPOSE 80

ENTRYPOINT ["/entrypoint.sh"]
