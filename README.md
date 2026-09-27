# axum_server

A newsletter delivery API based on [Zero to Production in Rust](https://www.zero2prod.com).
Written January–September 2024, without AI tools.

## Changes from the book

- axum instead of actix-web
- Diesel (async) instead of sqlx
- Tera for HTML rendering
- tower middleware and tracing
- A Nix flake (with direnv) that sets up the whole toolchain
- Some restructuring of modules along the way

## Status

Stops after idempotency for the publish-newsletter route; the later chapters
aren't implemented. In 2026 I did a short cleanup pass with Claude Code:
secrets moved out of the base config, CI fixed, clippy warnings resolved.

## Running it

Requires Postgres and Redis. `scripts/init_db.sh` and `scripts/init_redis.sh`
start them in Docker (set `SKIP_DOCKER=1` to use local instances) and run the migrations.

    cargo run
    cargo test

A seed migration creates a local-only admin user: `admin` / `adminpassword`.

Production reads secrets from environment variables (`APP_APPLICATION__HMAC_SECRET`,
database credentials); see `spec.yaml`.
