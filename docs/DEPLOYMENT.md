# Deployment Guide

This document explains how to deploy the DBridge (ABChat) project using Docker Compose.

## Prerequisites

-   [Docker](https://docs.docker.com/get-docker/)
-   [Docker Compose](https://docs.docker.com/compose/install/)

## Nginx Architecture

The project uses **Nginx** as a reverse proxy to manage internal and external routing.
-   **Frontend**: Served at `http://localhost/` (root).
-   **Backend**: Routed to `http://localhost/api` (REST and WebSockets).

### Important Note on API Prefixing
To ensure proper routing:
1.  **Backend Routes**: All API and WebSocket routes in the backend must be prefixed with `/api` (e.g., `/api/auth/login`, `/api/ws/{id}`).
2.  **Frontend Config**: The `VITE_API_BASE_URL` and `VITE_WS_BACKEND_URL` environment variables should include the `/api` suffix.

If you do not prefix the backend routes, requests from the frontend to `/api` will return 404 from the backend.

### Quick Start

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/your-repo/abchat.git
    cd abchat
    ```

2.  **Configure Environment Variables:**
    Copy the example environment file and update it with your secrets and configuration.
    ```bash
    cp .env.example .env
    ```
    Open `.env` and fill in the required values. Ensure the frontend URLs end in `/api`.

3.  **Build and Start the services:**
    ```bash
    docker compose up -d --build
    ```

4.  **Access the applications:**
    -   **Frontend:** [http://localhost](http://localhost)
    -   **Backend (via Proxy):** [http://localhost/api](http://localhost/api)

## Environment Variables

The project uses a single `.env` file at the root to manage configuration across all components.

### Shared / Infrastructure

| Variable | Description | Example |
|----------|-------------|---------|
| `HTTP_PORT` | The external port Nginx will listen on for incoming traffic. | `80` |
| `POSTGRES_USER` | PostgreSQL username. | `abchat` |
| `POSTGRES_PASSWORD` | PostgreSQL password. | `secure_password` |
| `POSTGRES_DB` | PostgreSQL database name. | `abchat` |

### Backend (`backend/`)

| Variable | Description | Example |
|----------|-------------|---------|
| `JWT_SECRET` | Secret key for signing and verifying JSON Web Tokens. | `my_super_secret_key` |
| `ADMIN_USERNAME` | (Optional) Username for the administrative user. | `admin` |
| `ADMIN_PASSWORD_HASH` | (Optional) Argon2 hash of the administrative user's password. Use **single quotes** for the value to avoid issues with shell expansion of '$'. | `'$argon2id$v=19$m=...'` |
| `VAPID_PUBLIC_KEY` | Public key for Web Push notifications. | `B...` |
| `VAPID_PRIVATE_KEY` | Private key for Web Push notifications. | `...` |

You can generate the Argon2 hash for your admin password by running the following command in the `backend/` directory:
```bash
cargo run --bin hash
```

To generate VAPID keys, you can use various CLI tools or online generators. Ensure they are in the correct Base64URL format.

### Frontend (`frontend/`)

Note: Frontend variables are built into the static assets and must be present during the build process.

| Variable | Description | Example |
|----------|-------------|---------|
| `VITE_WS_BACKEND_URL` | WebSocket URL for real-time communication. | `ws://localhost/api` |
| `VITE_API_BASE_URL` | Base URL for REST API calls to the backend. | `http://localhost/api` |

### Bot (`bot/`)

| Variable | Description | Example |
|----------|-------------|---------|
| `DISCORD_TOKEN` | Authentication token for the Discord bot. | `MTIzNDU2Nzg5...` |
| `DISCORD_CHANNEL_ID` | The ID of the Discord channel to bridge. | `1234567890` |
| `ABCHAT_CHANNEL_ID` | The internal DBridge channel ID to bridge. | `1234567890` |
| `LOG_LEVEL` | Logging verbosity (`error`, `warn`, `info`, `debug`). | `info` |
| `NODE_ENV` | Environment mode (`development`, `production`). | `production` |


## Persistence

Data is persisted using Docker volumes:
-   `postgres_data`: Stores PostgreSQL database files.
-   `valkey_data`: Stores Valkey (Redis) data.

## Management

### View Logs
```bash
docker compose logs -f [service_name]
```

### Stop Services
```bash
docker compose down
```

### Update Services
```bash
git pull
docker compose up -d --build
```
