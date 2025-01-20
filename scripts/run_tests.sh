#!/bin/bash

# 设置Python路径
export PYTHONPATH=src/backend:$PYTHONPATH

# 创建测试结果目录
mkdir -p test_results/coverage

# 运行OKX交易所集成测试
echo "开始运行OKX交易所集成测试..."

# REST API客户端测试
echo "测试REST API客户端..."
pytest src/backend/data_service/exchanges/okx/tests/test_rest_client.py \
    --cov=src/backend/data_service/exchanges/okx \
    --cov-report=html:test_results/coverage/rest \
    --cov-report=term-missing \
    -v

# WebSocket客户端测试
echo "测试WebSocket客户端..."
pytest src/backend/data_service/exchanges/okx/tests/test_websocket.py \
    --cov=src/backend/data_service/exchanges/okx \
    --cov-report=html:test_results/coverage/ws \
    --cov-report=term-missing \
    -v

# 如果测试失败则退出
if [ $? -ne 0 ]; then
    echo "测试失败!"
    exit 1
fi

echo "测试完成，覆盖率报告已生成在 test_results/coverage 目录" 