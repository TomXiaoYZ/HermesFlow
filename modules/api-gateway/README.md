# API Gateway Service

HermesFlow API 网关服务 - 使用 Spring Boot 构建的统一入口网关。

## 🚀 功能

- ✅ 健康检查端点 (`/health`)
- ✅ Spring Boot Actuator 集成

## 🛠️ 技术栈

- **语言**: Java 21
- **框架**: Spring Boot 3.2.0

## 📦 本地开发

```bash
./mvnw spring-boot:run
```

## 🧪 测试

```bash
./mvnw test
```

## 🐳 Docker

```bash
docker build -t api-gateway:latest .
docker run -p 8000:8000 api-gateway:latest
```
