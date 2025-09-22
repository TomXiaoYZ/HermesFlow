# HermesFlow 用户管理 Dockerfile
FROM openjdk:21-jre-slim

LABEL maintainer="HermesFlow Team"
LABEL module="user-management"
LABEL version="1.0.0"

# 设置工作目录
WORKDIR /app

# 安装系统依赖
RUN apt-get update && apt-get install -y \
    curl \
    && rm -rf /var/lib/apt/lists/*

# 复制构建产物
COPY target/*.jar app.jar

# 创建非root用户
RUN useradd -m -u 1000 appuser && chown -R appuser:appuser /app
USER appuser

# 健康检查
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8010/health || exit 1

# 暴露端口
EXPOSE 8010

# JVM优化参数
ENV JAVA_OPTS="-Xms256m -Xmx512m -XX:+UseG1GC -XX:+UseContainerSupport"

# 启动命令
ENTRYPOINT ["sh", "-c", "java $JAVA_OPTS -jar app.jar"]

