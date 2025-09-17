FROM openjdk:21-jdk-slim

# 设置工作目录
WORKDIR /app

# 安装必要的工具
RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

# 复制Maven配置文件
COPY pom.xml .

# 复制源代码
COPY src ./src

# 安装Maven
RUN apt-get update && apt-get install -y maven && rm -rf /var/lib/apt/lists/*

# 构建应用
RUN mvn clean package -DskipTests

# 创建日志目录
RUN mkdir -p /app/logs

# 暴露端口
EXPOSE 8081

# 设置JVM参数
ENV JAVA_OPTS="-Xms512m -Xmx1024m -XX:+UseG1GC"

# 启动应用
CMD ["sh", "-c", "java $JAVA_OPTS -jar target/permission-management-1.0.0.jar"] 