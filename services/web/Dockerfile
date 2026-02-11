FROM node:20-alpine AS builder

WORKDIR /app

# Copy package files
COPY services/web/package.json services/web/package-lock.json* ./

# Install dependencies
RUN npm ci

# Copy source code
COPY services/web .

# Expose port
EXPOSE 3000

# Run in development mode
CMD ["npm", "run", "dev"]
