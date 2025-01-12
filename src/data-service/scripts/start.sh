#!/bin/bash

# 确保脚本在出错时退出
set -e

# 启动依赖服务
echo "Starting dependencies..."
cd ../../deploy/local
docker-compose up -d

# 等待服务就绪
echo "Waiting for services to be ready..."
sleep 10

# 启动数据服务
echo "Starting data service..."
cd ../../src/data-service
poetry run python -m app.main 