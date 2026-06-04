# ── Stage 1: build the React frontend ────────────────────────────────────────
FROM node:20-slim AS frontend
WORKDIR /app
COPY package.json package-lock.json ./
RUN npm ci
COPY index.html tsconfig*.json vite.config.ts ./
COPY src ./src
COPY resources ./resources
RUN npm run build

# ── Stage 2: build the Rust server ───────────────────────────────────────────
FROM rust:1.81-slim AS backend
WORKDIR /app

# Install system deps for rusqlite (bundled) and reqwest (rustls)
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

COPY src-tauri/Cargo.toml src-tauri/Cargo.lock* ./src-tauri/
COPY src-tauri/build.rs ./src-tauri/
COPY src-tauri/src ./src-tauri/src
COPY src-tauri/capabilities ./src-tauri/capabilities
COPY resources ./resources

WORKDIR /app/src-tauri
RUN cargo build --release --bin job-search-server

# ── Stage 3: final minimal image ─────────────────────────────────────────────
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=backend /app/src-tauri/target/release/job-search-server ./
COPY --from=frontend /app/dist ./dist

# Railway mounts a persistent volume at /data
ENV DATA_DIR=/data
ENV DIST_DIR=/app/dist
ENV RUST_LOG=info

EXPOSE 3000
CMD ["./job-search-server"]
