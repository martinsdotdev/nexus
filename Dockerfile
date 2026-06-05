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
RUN --mount=type=cache,id=pnpm-store,target=/root/.local/share/pnpm/store \
    pnpm install --frozen-lockfile
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
# Compile + cache all dependencies first (busts only on Cargo.toml / Cargo.lock change).
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo chef cook --release --recipe-path recipe.json
COPY Cargo.toml Cargo.lock ./
COPY crates crates
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo build --release --locked -p nexus-server
# Output: /src/target/release/nexus-server

# ---- Stage 3: minimal runtime ---------------------------------------------
# Run as root (the default distroless user), not :nonroot. The relay writes its
# snapshot to the mounted volume on startup; a freshly provisioned Railway/Docker
# volume at the data dir is root-owned, so a non-root uid hits EACCES on the first
# write (verified locally). Root writes any volume regardless of how the platform
# provisions it; acceptable for a hosted single-tenant relay (Railway isolates the
# container). The local-first binary is unaffected (this is the hosted image only).
FROM gcr.io/distroless/cc-debian12 AS runtime
WORKDIR /srv
COPY --from=rust-build /src/target/release/nexus-server /usr/local/bin/nexus
COPY --from=ui /app/packages/app/build /srv/ui
EXPOSE 7777
# Railway overrides this via deploy.startCommand (run in a shell, which expands
# $PORT and the volume mount path). This default is for a plain `docker run`.
ENTRYPOINT ["/usr/local/bin/nexus"]
CMD ["serve", "--host", "0.0.0.0", "--static-dir", "/srv/ui", "--data-dir", "/data"]
