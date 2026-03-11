# Build stage
FROM node:22-alpine AS builder

WORKDIR /usr/src/frontend

# Install pnpm
RUN corepack enable pnpm

# Copy package files
COPY package.json pnpm-lock.yaml* ./

# Install dependencies
RUN --mount=type=cache,target=/root/.local/share/pnpm/store \
    pnpm install --frozen-lockfile

# Copy the source code
COPY . .

# Set build arguments for Vite
ARG VITE_WS_BACKEND_URL
ARG VITE_API_BASE_URL
ENV VITE_WS_BACKEND_URL=$VITE_WS_BACKEND_URL
ENV VITE_API_BASE_URL=$VITE_API_BASE_URL

# Build the application
RUN pnpm build

# Runtime stage
FROM nginx:stable-alpine

RUN apk add nodejs
COPY --from=builder /usr/src/frontend/scripts/write-env.js /write-env.js
COPY entrypoint.sh /entrypoint.sh

# Copy the built files from the build stage
COPY --from=builder /usr/src/frontend/dist /usr/share/nginx/html

# Add custom nginx configuration to support SPA routing
RUN printf 'server {\n    listen 80;\n    location / {\n        root /usr/share/nginx/html;\n        index index.html index.htm;\n        try_files $uri $uri/ /index.html;\n    }\n}\n' > /etc/nginx/conf.d/default.conf

EXPOSE 80

CMD ["/entrypoint.sh"]
