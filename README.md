# litellm-rs

A high-performance AI Gateway written in Rust, providing OpenAI-compatible APIs with intelligent routing, load balancing, and enterprise features.

## Features

- OpenAI-compatible API endpoints
- Multi-provider support (OpenAI, Anthropic, Google, Azure, etc.)
- Intelligent load balancing and routing
- Request/response caching
- Rate limiting and quotas
- Authentication and API key management
- Metrics and observability (Prometheus, OpenTelemetry)
- Database support (PostgreSQL, SQLite)
- Redis caching
- S3-compatible object storage

## Installation

```bash
cargo install litellm-rs
```

## Quick Start

```bash
# Run the gateway
gateway --config config.yaml
```

## Configuration

See `config/` directory for example configurations.

## License

MIT License
