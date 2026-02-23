# Build stage
FROM node:20-alpine AS builder

WORKDIR /app

# Copy package files
COPY services/web/package.json services/web/package-lock.json* ./

# Install dependencies
RUN npm ci

# Copy source code
COPY services/web .

# Override .env.local so rewrites use Docker-internal gateway hostname
ENV API_URL=http://gateway:8080

# Build for production (output: standalone)
RUN npm run build

# Runtime stage
FROM node:20-alpine AS runner

WORKDIR /app

ENV NODE_ENV=production

# Create non-root user
RUN addgroup --system --gid 1001 nodejs && \
    adduser --system --uid 1001 nextjs

# Copy standalone output and static assets
COPY --from=builder /app/.next/standalone ./
COPY --from=builder /app/.next/static ./.next/static
COPY --from=builder /app/public ./public

# Set correct ownership
RUN chown -R nextjs:nodejs /app

USER nextjs

EXPOSE 3000

ENV PORT=3000
ENV HOSTNAME="0.0.0.0"

CMD ["node", "server.js"]
