# Multi-stage build for LiteLLM-RS workspace split
FROM rust:1.87-slim AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY apps ./apps
COPY config ./config

RUN cargo build --release -p gateway -p google-gateway

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m -u 1001 appuser
WORKDIR /app

COPY --from=builder /app/target/release/gateway /usr/local/bin/gateway
COPY --from=builder /app/target/release/google-gateway /usr/local/bin/google-gateway
COPY --from=builder /app/config ./config

RUN chown -R appuser:appuser /app
USER appuser

EXPOSE 8000

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8000/health || exit 1

ENV GATEWAY_CONFIG=config/gateway.yaml
CMD ["gateway"]
