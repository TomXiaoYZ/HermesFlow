#!/bin/bash
# HermesFlow 一键部署与运维脚本
# 用法示例：
#   ./scripts/deploy.sh --env local up
#   ./scripts/deploy.sh --env local down
#   ./scripts/deploy.sh --env local status
#   ./scripts/deploy.sh --env local logs
#   ./scripts/deploy.sh --env local clean
#   ./scripts/deploy.sh --env dev up

set -e

ENV=local
CMD=up

# 解析参数
while [[ $# -gt 0 ]]; do
  case $1 in
    --env)
      ENV="$2"
      shift 2
      ;;
    up|down|status|logs|clean|help)
      CMD="$1"
      shift
      ;;
    *)
      echo "[ERROR] 未知参数: $1"
      echo "用法: ./deploy.sh --env [local|dev|prod] [up|down|status|logs|clean|help]"
      exit 1
      ;;
  esac
done

show_help() {
  echo "HermesFlow 一键部署与运维脚本"
  echo "用法: ./deploy.sh --env [local|dev|prod] [up|down|status|logs|clean|help]"
  echo "  up      : 启动所有服务 (docker compose，包括 nacos、gateway、api-gateway、基础设施等)"
  echo "  down    : 停止所有服务"
  echo "  status  : 查看服务状态"
  echo "  logs    : 查看所有服务日志（Ctrl+C退出）"
  echo "  clean   : 停止并清理所有容器和数据卷"
  echo "  help    : 显示帮助信息"
  echo "  dev/prod 环境暂未实现自动部署，后续可扩展上传、远程部署等逻辑。"
}

if [[ "$CMD" == "help" ]]; then
  show_help
  exit 0
fi

if [[ "$ENV" == "local" ]]; then
  case $CMD in
    up)
      echo "[INFO] 启动所有服务 (docker-compose，包括 nacos、gateway、api-gateway、基础设施等)..."
      docker-compose -f docker-compose.local.yml up -d
      ;;
    down)
      echo "[INFO] 停止所有服务..."
      docker-compose -f docker-compose.local.yml down
      ;;
    status)
      docker-compose -f docker-compose.local.yml ps
      ;;
    logs)
      docker-compose -f docker-compose.local.yml logs -f
      ;;
    clean)
      echo "[INFO] 停止并清理所有容器和数据卷..."
      docker-compose -f docker-compose.local.yml down -v
      ;;
    *)
      show_help
      ;;
  esac
  exit 0
fi

# dev/prod 环境下的后续逻辑（如前端/后端编译、打包、上传、启动）
# ... 