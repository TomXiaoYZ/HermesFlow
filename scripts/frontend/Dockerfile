# HermesFlow 前端 Dockerfile
FROM node:18-alpine as builder

LABEL maintainer="HermesFlow Team"
LABEL module="frontend"
LABEL version="1.0.0"

# 设置工作目录
WORKDIR /app

# 复制package文件
COPY package*.json ./

# 安装依赖
RUN npm ci --only=production

# 复制源代码
COPY . .

# 构建应用
RUN npm run build

# 生产阶段
FROM nginx:alpine

# 复制构建产物
COPY --from=builder /app/dist /usr/share/nginx/html

# 复制nginx配置
COPY --from=builder /app/nginx.conf /etc/nginx/conf.d/default.conf 2>/dev/null || true

# 创建nginx配置（如果不存在）
RUN if [ ! -f /etc/nginx/conf.d/default.conf ]; then \
    echo 'server { \
        listen 80; \
        location / { \
            root /usr/share/nginx/html; \
            index index.html index.htm; \
            try_files $uri $uri/ /index.html; \
        } \
        location /api/ { \
            proxy_pass http://api-gateway:8000/; \
            proxy_set_header Host $host; \
            proxy_set_header X-Real-IP $remote_addr; \
        } \
    }' > /etc/nginx/conf.d/default.conf; \
fi

# 健康检查
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost/ || exit 1

# 暴露端口
EXPOSE 80

# 启动nginx
CMD ["nginx", "-g", "daemon off;"]

