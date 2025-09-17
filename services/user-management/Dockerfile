# 使用官方的OpenJDK 21镜像作为基础镜像
FROM openjdk:21-jdk-slim

# 设置工作目录
WORKDIR /app

# 创建非root用户
RUN groupadd -r hermesflow && useradd -r -g hermesflow hermesflow

# 复制JAR文件
COPY target/user-management-1.0.0.jar app.jar

# 创建日志目录
RUN mkdir -p /app/logs && chown -R hermesflow:hermesflow /app

# 切换到非root用户
USER hermesflow

# 暴露端口
EXPOSE 18010

# 设置JVM参数
ENV JAVA_OPTS="-Xms512m -Xmx1024m -XX:+UseG1GC -XX:+UseContainerSupport"

# 健康检查
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
  CMD curl -f http://localhost:18010/health || exit 1

# 启动应用
ENTRYPOINT ["sh", "-c", "java $JAVA_OPTS -jar app.jar"] 