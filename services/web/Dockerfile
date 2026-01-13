# Build Stage
FROM node:18-alpine AS builder

WORKDIR /app
COPY services/web/package.json ./services/web/
# Assuming lockfile might not exist yet, or copy if exists
# COPY services/web/package-lock.json ./services/web/

WORKDIR /app/services/web
RUN npm install

# Copy source code
COPY services/web ./

# Build
# Note: VITE_API_BASE_URL should be passed as build argument or handled at runtime
ARG VITE_API_URL
ENV VITE_API_URL=$VITE_API_URL
RUN npm run build

# Production Stage
FROM nginx:alpine

# Copy built assets from builder stage
COPY --from=builder /app/services/web/dist /usr/share/nginx/html

# Copy custom nginx config if we had one (using default for now, fine for SPA if configured)
# For SPA routing support, we need a custom nginx config
RUN echo 'server { \
    listen 80; \
    location / { \
        root /usr/share/nginx/html; \
        index index.html index.htm; \
        try_files $uri $uri/ /index.html; \
    } \
    location /api/ { \
        proxy_pass http://gateway:8080/; \
        proxy_set_header Host $host; \
        proxy_set_header X-Real-IP $remote_addr; \
    } \
}' > /etc/nginx/conf.d/default.conf

EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
