# Build stage
FROM rust:1-bookworm AS builder

WORKDIR /usr/src/backend

# Copy the source code
COPY . .

# Build the application
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/usr/src/backend/target \
    cargo build --release && \
    cp target/release/backend /usr/local/bin/backend

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/local/bin

# Copy the binary from the build stage
COPY --from=builder /usr/local/bin/backend .

# Expose the default port
EXPOSE 8080

# Run the binary
CMD ["./backend"]
