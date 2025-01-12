# 使用 Python 3.8 作为基础镜像
FROM python:3.8-slim

# 设置工作目录
WORKDIR /app

# 安装 Poetry
RUN pip install poetry

# 复制项目文件
COPY pyproject.toml poetry.lock ./
COPY app ./app

# 配置 Poetry
RUN poetry config virtualenvs.create false \
    && poetry install --no-dev --no-interaction --no-ansi

# 设置环境变量
ENV PYTHONPATH=/app
ENV PYTHONUNBUFFERED=1

# 暴露端口
EXPOSE 8000 8001

# 启动应用
CMD ["poetry", "run", "python", "-m", "app.main"] 