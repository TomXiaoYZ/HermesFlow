#!/bin/bash

# HermesFlow User Management Service Docker Entrypoint
# 用于在Docker容器中启动用户管理服务

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 日志函数
log_info() {
    echo -e "${GREEN}[INFO]${NC} $(date '+%Y-%m-%d %H:%M:%S') - $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $(date '+%Y-%m-%d %H:%M:%S') - $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $(date '+%Y-%m-%d %H:%M:%S') - $1"
}

log_debug() {
    echo -e "${BLUE}[DEBUG]${NC} $(date '+%Y-%m-%d %H:%M:%S') - $1"
}

# 等待数据库连接
wait_for_database() {
    log_info "等待数据库连接..."
    
    local max_attempts=30
    local attempt=1
    
    while [ $attempt -le $max_attempts ]; do
        if pg_isready -h "${SPRING_DATASOURCE_HOST:-postgres}" -p "${SPRING_DATASOURCE_PORT:-5432}" -U "${SPRING_DATASOURCE_USERNAME:-hermesflow}" > /dev/null 2>&1; then
            log_info "数据库连接成功"
            return 0
        fi
        
        log_warn "数据库连接失败，重试 $attempt/$max_attempts"
        sleep 2
        attempt=$((attempt + 1))
    done
    
    log_error "数据库连接超时"
    exit 1
}

# 等待Redis连接
wait_for_redis() {
    log_info "等待Redis连接..."
    
    local max_attempts=30
    local attempt=1
    
    while [ $attempt -le $max_attempts ]; do
        if redis-cli -h "${SPRING_DATA_REDIS_HOST:-redis}" -p "${SPRING_DATA_REDIS_PORT:-6379}" -a "${SPRING_DATA_REDIS_PASSWORD:-hermesflow_redis_password}" ping > /dev/null 2>&1; then
            log_info "Redis连接成功"
            return 0
        fi
        
        log_warn "Redis连接失败，重试 $attempt/$max_attempts"
        sleep 2
        attempt=$((attempt + 1))
    done
    
    log_error "Redis连接超时"
    exit 1
}

# 检查环境变量
check_environment() {
    log_info "检查环境变量..."
    
    # 必需的环境变量
    local required_vars=(
        "SPRING_DATASOURCE_URL"
        "SPRING_DATASOURCE_USERNAME"
        "SPRING_DATASOURCE_PASSWORD"
        "SPRING_DATA_REDIS_HOST"
        "JWT_SECRET"
    )
    
    for var in "${required_vars[@]}"; do
        if [ -z "${!var}" ]; then
            log_error "缺少必需的环境变量: $var"
            exit 1
        fi
    done
    
    log_info "环境变量检查完成"
}

# 创建日志目录
create_log_directory() {
    log_info "创建日志目录..."
    mkdir -p /app/logs
    chmod 755 /app/logs
    log_info "日志目录创建完成"
}

# 显示启动信息
show_startup_info() {
    log_info "=========================================="
    log_info "HermesFlow User Management Service"
    log_info "Version: 1.0.0"
    log_info "Profile: ${SPRING_PROFILES_ACTIVE:-docker}"
    log_info "Port: 8080"
    log_info "=========================================="
}

# 启动应用
start_application() {
    log_info "启动用户管理服务..."
    
    # 设置JVM参数
    export JAVA_OPTS="${JAVA_OPTS:--Xmx512m -Xms256m -XX:+UseG1GC -XX:+UseContainerSupport -XX:MaxRAMPercentage=75.0}"
    
    # 启动Spring Boot应用
    exec java $JAVA_OPTS \
        -Dspring.profiles.active=${SPRING_PROFILES_ACTIVE:-docker} \
        -Dfile.encoding=UTF-8 \
        -Djava.security.egd=file:/dev/./urandom \
        -jar /app/app.jar
}

# 主函数
main() {
    show_startup_info
    check_environment
    create_log_directory
    wait_for_database
    wait_for_redis
    start_application
}

# 信号处理
trap 'log_info "接收到停止信号，正在关闭服务..."; exit 0' SIGTERM SIGINT

# 执行主函数
main "$@" 