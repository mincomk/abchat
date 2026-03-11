# DBridge Project Guide

This document serves as a global guide for AI agents working on the DBridge project.

## Project Overview
DBridge is a message bridging engine designed to connect multiple communication platforms (e.g., Discord, Web). It follows a master-secondary architecture for message channels.

## Project Structure
- `backend/`: The core bridging engine written in Rust.
    - `src/types/`: API DTOs (`dto.rs`) and internal models (`model.rs`).
    - `src/api/routes/`: Route handlers (e.g., `admin.rs`, `auth.rs`).
    - `src/api/router.rs`: Route registration using `utoipa`.
    - `src/service/persistence/`: Data persistence layer.
        - `postgres.rs`: SQL implementation using `sqlx`.
        - `in_memory.rs`: DashMap implementation for testing/dev.
    - `src/error.rs`: Centralized error handling (`AppError`, `UserError`).
- `bot/`: Discord bot integration written in TypeScript.
- `frontend/`: TypeScript/React-based web UI.
    - `src/api/dbridge-api.ts`: Central API client (`DBridgeClient`).
    - `src/pages/`: Main screen components.
    - `src/components/`: Reusable UI elements (ui/, chat/, admin/).
    - `src/i18n/locales/`: Translation files (`en.json`, `ko.json`).
    - `src/App.tsx`: Main entry point with React Router v7 navigation.
- `docker-compose.yml`: Main orchestration for development and deployment.

## Key Technologies
- **Backend**: Rust, `axum` (Web), `tokio` (Async), `sqlx` (PostgreSQL), `utoipa` (OpenAPI), `argon2` (Hashing).
- **Frontend**: TypeScript, React, **React Router v7**, Vite, Tailwind CSS.

## Persistence Layer
The backend uses a `Persistence` trait (`src/service/persistence.rs`) to abstract data operations. 
- **Persistence Layer**: The `save_user` method handles both insertion and updates (via `ON CONFLICT` in Postgres). 
- **Push Notifications**:
    - **Backend**: `notifications.rs` handles VAPID key delivery, subscription storage, and user settings.
    - **NotificationService**: Central service for sending Web Push notifications using `web-push` crate. Supports targeted notifications based on user mentions.
    - **NotificationBroker**: Chat broker that listens for messages and triggers notifications for `All` mode users and `Critical` mode users (when mentioned).
    - **Persistence**: `get_subscriptions_by_mode` allows filtering subscriptions by user notification preferences.
    - **Frontend**: `notifications.ts` bridges the browser `PushManager` with the `DBridgeClient`.
    - **Environment**: Requires `VAPID_PUBLIC_KEY` and `VAPID_PRIVATE_KEY` in the backend `.env`.
- **Modifying Schema**: Update `init_db` in `postgres.rs` and the `User` struct/trait methods accordingly.

## Feature Development Workflow

### 1. Backend API & Logic
1.  **DTO**: Define or update request/response structures in `src/types/dto.rs`.
2.  **Error Handling**: If needed, add new error variants to `UserError` in `src/error.rs`.
3.  **Persistence**: Update the `Persistence` trait and its implementations if data storage requirements change.
4.  **Handlers**: Implement the logic in a new or existing module within `src/api/routes/`.
5.  **Routing**: Register the handler in `src/api/router.rs` using `utoipa::path`.
6.  **Verify**: Run `cargo check` in the `backend` directory.

### 2. Frontend Integration & UI
1.  **API Client**: Add new methods to `DBridgeClient` in `src/api/dbridge-api.ts` to match the backend endpoints.
2.  **Translations**: Add any new UI labels to `src/i18n/locales/en.json` and `ko.json`.
3.  **Components**: Extract complex UI logic into separate components within `src/components/`.
4.  **Routing**: Update `src/App.tsx` using React Router v7 (`Routes`, `Route`, `useNavigate`) for navigation and access control.
5.  **Verify**: Run `pnpm build` (or `npm run build`) in the `frontend` directory.

### 3. Adding Backend Configuration
1.  **Config**: Add the new field to `AppConfig` in `backend/src/config.rs`.
2.  **State**: Add the new field to `AppState` in `backend/src/state.rs`.
3.  **Main**: Update `backend/src/main.rs` to load the field from `AppConfig` and pass it to `AppState`.
4.  **Environment**: Add the new variable to `.env` and `.env.example`.
5.  **Deployment**: Update `docker-compose.yml` to pass the environment variable if needed.

## How to Update this file
Agents should update `AGENTS.md` whenever they:
1.  Discover new architectural patterns or significant components.
2.  Add new technologies or major dependencies.
3.  Establish new coding conventions or best practices.
4.  Find crucial information that would help future agents understand the codebase faster.

To update:
1.  Read the current `AGENTS.md`.
2.  Incorporate the new information into the relevant section or add a new one.
3.  Ensure the guide remains concise and actionable.
