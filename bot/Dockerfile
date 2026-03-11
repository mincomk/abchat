# Build stage
FROM node:22-alpine AS builder

# Install build dependencies for potential native addons
RUN apk add --no-cache python3 make g++

WORKDIR /usr/src/bot

# Install pnpm
RUN npm install -g pnpm@10.26.1

# Copy package files
COPY package.json pnpm-lock.yaml ./

# Install dependencies
RUN --mount=type=cache,target=/root/.local/share/pnpm/store \
    pnpm install --frozen-lockfile

# Copy the source code
COPY . .

# Build the application
RUN pnpm build

# Runtime stage
FROM node:22-alpine

WORKDIR /usr/src/bot

# Install pnpm for runtime if needed (optional if using node directly)
RUN npm install -g pnpm@10.26.1

# Copy package files
COPY package.json pnpm-lock.yaml ./

# Install production dependencies only
RUN --mount=type=cache,target=/root/.local/share/pnpm/store \
    pnpm install --prod --frozen-lockfile

# Copy the built files from the build stage
COPY --from=builder /usr/src/bot/dist ./dist

# Set the entry point
CMD ["node", "dist/index.js"]
