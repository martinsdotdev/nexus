# syntax=docker/dockerfile:1
#
# Hosted build of the Nexus relay (nexus-server) for Railway and similar PaaS.
# One binary serves the built SvelteKit UI as static files and the /sync WebSocket.
# Three stages: Node/pnpm builds the UI, cargo-chef release-builds the Rust binary
# with cached dependency layers, a distroless cc runtime carries only the artifacts.
# Env expansion ($PORT, volume path) is done by the platform start command (see
# railway.toml); distroless has no shell, so the image CMD is only a `docker run`
# default. Local-first use is unaffected: the binary still defaults to 127.0.0.1.

# ---- Stage 1: build the SvelteKit static UI -------------------------------
FROM node:22-bookworm-slim AS ui
WORKDIR /app
ENV CI=1
# No packageManager field in the repo, so pin pnpm explicitly (10 reads v9 lockfiles).
RUN corepack enable && corepack prepare pnpm@10 --activate
# Lockfile + workspace manifest live at the repo root; app manifest under packages/app.
COPY pnpm-lock.yaml pnpm-workspace.yaml package.json ./
COPY packages/app/package.json packages/app/package.json
RUN pnpm install --frozen-lockfile
COPY packages/app packages/app
RUN pnpm -F app build
# Output: /app/packages/app/build (adapter-static, 200.html SPA fallback)

# ---- Stage 2a: cargo-chef dependency recipe -------------------------------
FROM rust:1.90-bookworm AS chef
RUN cargo install cargo-chef --locked
WORKDIR /src

FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY crates crates
RUN cargo chef prepare --recipe-path recipe.json

# ---- Stage 2b: build nexus-server (release) -------------------------------
FROM chef AS rust-build
COPY --from=planner /src/recipe.json recipe.json
# Compile all dependencies first. This is a cached Docker layer that busts only when
# Cargo.toml / Cargo.lock (the recipe) changes, which is cargo-chef's main speedup.
RUN cargo chef cook --release --recipe-path recipe.json
COPY Cargo.toml Cargo.lock ./
COPY crates crates
RUN cargo build --release --locked -p nexus-server
# Output: /src/target/release/nexus-server

# ---- Stage 3: minimal runtime ---------------------------------------------
# debian-slim, NOT distroless: the launch command needs a real shell to expand the
# Railway-injected $PORT and the volume mount path. A distroless image has no /bin/sh,
# so "$PORT" never expands and the container fails its healthcheck (verified: the
# distroless deploy never became reachable). Runs as root by default, so it can write
# the mounted volume (a freshly provisioned volume is root-owned; a non-root uid hits
# EACCES on the first snapshot write). glibc base matches the dynamically-linked
# binary; the relay makes no outbound TLS calls, so no ca-certificates are needed.
# The local-first binary is unaffected; this is the hosted image only.
FROM debian:bookworm-slim AS runtime
WORKDIR /srv
COPY --from=rust-build /src/target/release/nexus-server /usr/local/bin/nexus
COPY --from=ui /app/packages/app/build /srv/ui
EXPOSE 7777
# Shell-form so the container itself expands $PORT (Railway-injected) and the volume
# mount path; `exec` makes the binary PID 1 so it receives signals. This is the single
# launch path: railway.toml sets no startCommand, so Railway runs this CMD as-is.
CMD ["sh", "-c", "exec /usr/local/bin/nexus serve --host 0.0.0.0 --port ${PORT:-7777} --static-dir /srv/ui --data-dir ${RAILWAY_VOLUME_MOUNT_PATH:-/data}"]
