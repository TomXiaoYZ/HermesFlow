# User Management Service

HermesFlow 用户管理服务 - 使用 Spring Boot 构建的 RESTful API 服务。

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
docker build -t user-management:latest .
docker run -p 8010:8010 user-management:latest
```
