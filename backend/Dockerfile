# ── Stage 1: dependency cache ─────────────────────────────────────────────────
# Compile only dependencies first so Docker can cache this layer.
# The real source is added in stage 2, which only re-runs when src/** changes.
FROM rust:1-bookworm AS deps

WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs \
    && cargo build --release \
    && rm -rf src target/release/deps/backend* target/release/backend*

# ── Stage 2: full build ────────────────────────────────────────────────────────
FROM deps AS builder

COPY src ./src
RUN cargo build --release

# ── Stage 3: minimal runtime ───────────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl3 \
    ca-certificates \
    wget \
    && rm -rf /var/lib/apt/lists/*

# Dedicated non-root system user with no login shell
RUN useradd --system --no-create-home --shell /bin/false --uid 10001 appuser

WORKDIR /app

COPY --from=builder /app/target/release/backend ./backend
# Base config is baked in; override with a volume mount or APP__* env vars.
COPY config/app.yaml ./config/app.yaml

USER appuser

EXPOSE 3000

ENV RUST_LOG=backend=info,tower_http=warn

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://127.0.0.1:3000/api/v1/health || exit 1

CMD ["./backend"]
