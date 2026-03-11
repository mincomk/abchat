# DBridge Project Guide

This document serves as a global guide for AI agents working on the DBridge project.

## Project Overview
DBridge is a message bridging engine designed to connect multiple communication platforms (e.g., Discord, Web). It follows a master-secondary architecture for message channels.

## Project Structure
- `backend/`: The core bridging engine written in Rust.
    - `core/`: Shared types and logic (channels, providers, members, etc.).
    - `engine/`: The main orchestrator of platform integrations and message flows.
    - `auth/`: CLI tool to generate JWT tokens for the web backend.
    - `persistence/`: Data persistence layer (`InMemoryPersistence`, `PostgresPersistence`).
    - `pubsub/`: Message pubsub layer (`InMemoryPubSub` (using `pubsub-rs`), `RedisMessagePubSub`).
    - `provider/`: Integration with external platforms (Discord, Web).
- `bot/`: Discord bot integration written in TypeScript.
- `frontend/`: TypeScript/React-based web UI for DBridge.
- `signer/`: A separate Rust tool for JWT signing.
- `docs/`: Project documentation.
- `docker-compose.yml`: Main orchestration for development and deployment.

## Key Technologies
- **Backend**: Rust, `tokio` (async runtime), `serenity` (Discord), `axum` (Web), `serde` (serialization), `tracing` (logging), `jsonwebtoken` (JWT), `redis` (PubSub), `sqlx` (PostgreSQL), `valkey` (Infrastructure).
- **Frontend**: TypeScript, Vite, React (implied).

## Development Conventions
- **Language**: Rust for backend/tools, TypeScript for frontend.
- **Error Handling**: Use `Result` and `anyhow` (or similar) where appropriate. Main backend entry point uses `Box<dyn std::error::Error>`.
- **Async**: Use `tokio` for all async operations.
- **Logging**: Use `tracing` for instrumentation and logging.
- **Configuration**: Managed through `config.toml` and environment variables (using `dotenvy`).

## How to Update this file
Agents should update `AGENTS.md` whenever they:
1.  Discover new architectural patterns or significant components.
2.  Add new technologies or major dependencies.
3.  Establish new coding conventions or best practices.
4.  Find crucial information that would help future agents understand the codebase faster.
5.  Add or update environment variables. In this case, also update `docs/DEPLOYMENT.md`.

To update:
1.  Read the current `AGENTS.md`.
2.  Incorporate the new information into the relevant section or add a new one.
3.  Ensure the guide remains concise and actionable.
