# HermesFlow 数据引擎 Dockerfile
FROM python:3.12-slim

LABEL maintainer="HermesFlow Team"
LABEL module="data-engine"
LABEL version="1.0.0"

# 设置工作目录
WORKDIR /app

# 安装系统依赖
RUN apt-get update && apt-get install -y \
    curl \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# 复制依赖文件并安装Python包
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# 复制应用代码
COPY . .

# 创建非root用户
RUN useradd -m -u 1000 appuser && chown -R appuser:appuser /app
USER appuser

# 健康检查
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8001/health || exit 1

# 暴露端口
EXPOSE 8001

# 设置Python路径
ENV PYTHONPATH=/app

# 启动命令
CMD ["python", "-c", "from src.data import main; main.run()"]

