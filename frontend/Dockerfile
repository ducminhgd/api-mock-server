# ── dev stage ─────────────────────────────────────────────────────────────────
FROM cgr.dev/chainguard/node:latest-dev AS dev

WORKDIR /app

COPY package.json package-lock.json ./
RUN npm ci

EXPOSE 5173

CMD ["npm", "run", "dev", "--", "--host"]

# ── builder stage ─────────────────────────────────────────────────────────────
FROM cgr.dev/chainguard/node:latest-dev AS builder

WORKDIR /app

COPY package.json package-lock.json ./
RUN npm ci

COPY . .
RUN npm run build

# ── prod stage ────────────────────────────────────────────────────────────────
FROM cgr.dev/chainguard/nginx:latest AS prod

COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80
