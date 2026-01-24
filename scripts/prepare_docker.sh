#!/bin/bash
set -e

# Define build context directory
CTX_DIR=".build_ctx"

echo "Cleaning previous build context..."
rm -rf $CTX_DIR
mkdir -p $CTX_DIR

echo "Populating build context (excluding heavy artifacts)..."
# Copy services excluding heavy directories
rsync -av \
    --exclude '.git' \
    --exclude 'target' \
    --exclude 'node_modules' \
    --exclude '.next' \
    --exclude '__pycache__' \
    services $CTX_DIR/

# Copy infrastructure
rsync -av infrastructure $CTX_DIR/

# Copy .dockerignore
cp .dockerignore $CTX_DIR/

# Explicitly ensure Dockerfiles are present (Fail loudly if missing)
echo "Manually copying Dockerfiles..."
cp -v services/web/Dockerfile $CTX_DIR/services/web/
cp -v services/web/next.config.ts $CTX_DIR/services/web/
# Force copy all web config files as rsync seems to miss them
cp -v services/web/package.json $CTX_DIR/services/web/
cp -v services/web/package-lock.json $CTX_DIR/services/web/
cp -v services/web/tsconfig.json $CTX_DIR/services/web/

cp -v services/strategy-engine/Dockerfile $CTX_DIR/services/strategy-engine/
cp -v services/execution-engine/Dockerfile $CTX_DIR/services/execution-engine/
cp -v services/data-engine/Dockerfile $CTX_DIR/services/data-engine/

echo "Verifying content of .build_ctx/services/web:"
ls -la $CTX_DIR/services/web/


echo "Build context ready at $CTX_DIR (Size: $(du -sh $CTX_DIR | awk '{print $1}'))"
echo "You can now run: docker-compose up -d --build"
