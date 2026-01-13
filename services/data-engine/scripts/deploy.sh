#!/bin/bash
# Data Engine 部署脚本

set -e

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}Data Engine 部署脚本${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""

# 配置
AWS_REGION="us-west-2"
AWS_ACCOUNT_ID="739275455546"
ECR_REPO_NAME="data-engine"
ECS_CLUSTER_NAME="${1:-DifyOnAwsStack-ClusterEB0386A7-d4ySDF3hYoIw}"
SERVICE_NAME="data-engine"

# Use standard ECS execution role (created as ecsTaskExecutionRole)
ECS_EXECUTION_ROLE_ARN="arn:aws:iam::739275455546:role/ecsTaskExecutionRole"
ECS_SUBNET_1="subnet-0c05cbbffbf403cfb"
ECS_SUBNET_2="subnet-0a6c13d4e8a966c9b"
ECS_SECURITY_GROUP="sg-0eb44e4529b492928"
ECS_ASSIGN_PUBLIC_IP="DISABLED"

echo -e "${YELLOW}配置信息:${NC}"
echo "  AWS Region: $AWS_REGION"
echo "  AWS Account: $AWS_ACCOUNT_ID"
echo "  ECR Repository: $ECR_REPO_NAME"
echo "  ECS Cluster: $ECS_CLUSTER_NAME"
echo ""

# 步骤 1: 检查 ECR 仓库
echo -e "${YELLOW}[1/6] 检查 ECR 仓库...${NC}"
if aws ecr describe-repositories --region $AWS_REGION --repository-names $ECR_REPO_NAME &>/dev/null; then
    echo -e "${GREEN}✓ ECR 仓库已存在${NC}"
else
    echo -e "${YELLOW}  创建 ECR 仓库...${NC}"
    aws ecr create-repository \
        --region $AWS_REGION \
        --repository-name $ECR_REPO_NAME
    echo -e "${GREEN}✓ ECR 仓库创建成功${NC}"
fi
echo ""

# 步骤 2: 构建 Docker 镜像
echo -e "${YELLOW}[2/6] 构建 Docker 镜像...${NC}"
# ECS Fargate 默认运行在 x86_64；在 Apple Silicon 上需要显式构建 linux/amd64
docker build --platform linux/amd64 --no-cache -t $ECR_REPO_NAME:latest .
echo -e "${GREEN}✓ Docker 镜像构建完成${NC}"
echo ""

# 步骤 3: 登录 ECR
echo -e "${YELLOW}[3/6] 登录 ECR...${NC}"
aws ecr get-login-password --region $AWS_REGION | \
    docker login --username AWS --password-stdin \
    $AWS_ACCOUNT_ID.dkr.ecr.$AWS_REGION.amazonaws.com
echo -e "${GREEN}✓ ECR 登录成功${NC}"
echo ""

# 步骤 4: 标记和推送镜像
echo -e "${YELLOW}[4/6] 推送 Docker 镜像...${NC}"
docker tag $ECR_REPO_NAME:latest \
    $AWS_ACCOUNT_ID.dkr.ecr.$AWS_REGION.amazonaws.com/$ECR_REPO_NAME:latest

docker push $AWS_ACCOUNT_ID.dkr.ecr.$AWS_REGION.amazonaws.com/$ECR_REPO_NAME:latest
echo -e "${GREEN}✓ Docker 镜像推送成功${NC}"
echo ""

# 步骤 5: 创建 CloudWatch 日志组
echo -e "${YELLOW}[5/6] 创建 CloudWatch 日志组...${NC}"
if aws logs describe-log-groups --region $AWS_REGION --log-group-name-prefix "/ecs/$SERVICE_NAME" | grep -q "$SERVICE_NAME"; then
    echo -e "${GREEN}✓ CloudWatch 日志组已存在${NC}"
else
    aws logs create-log-group \
        --region $AWS_REGION \
        --log-group-name /ecs/$SERVICE_NAME
    echo -e "${GREEN}✓ CloudWatch 日志组创建成功${NC}"
fi
echo ""

# 步骤 6: 生成任务定义
echo -e "${YELLOW}[6/6] 生成 ECS 任务定义...${NC}"

# 获取安全组和子网信息
echo -e "${YELLOW}  获取网络配置...${NC}"

SUBNETS="[${ECS_SUBNET_1},${ECS_SUBNET_2}]"
SECURITY_GROUP="${ECS_SECURITY_GROUP}"

cat > task-definition.json <<EOF
{
  "family": "$SERVICE_NAME",
  "networkMode": "awsvpc",
  "requiresCompatibilities": ["FARGATE"],
  "cpu": "2048",
  "memory": "4096",
  "executionRoleArn": "$ECS_EXECUTION_ROLE_ARN",
  "containerDefinitions": [
    {
      "name": "$SERVICE_NAME",
      "image": "$AWS_ACCOUNT_ID.dkr.ecr.$AWS_REGION.amazonaws.com/$ECR_REPO_NAME:latest",
      "portMappings": [
        {
          "containerPort": 8080,
          "protocol": "tcp"
        }
      ],
      "environment": [
        {
          "name": "RUST_ENV",
          "value": "prod"
        },
        {
          "name": "DATA_ENGINE__SERVER__HOST",
          "value": "0.0.0.0"
        },
        {
          "name": "DATA_ENGINE__SERVER__PORT",
          "value": "8080"
        }
      ],
      "logConfiguration": {
        "logDriver": "awslogs",
        "options": {
          "awslogs-group": "/ecs/$SERVICE_NAME",
          "awslogs-region": "$AWS_REGION",
          "awslogs-stream-prefix": "ecs"
        }
      },
      "healthCheck": {
        "command": [
          "CMD-SHELL",
          "curl -f http://localhost:8080/metrics || exit 1"
        ],
        "interval": 30,
        "timeout": 5,
        "retries": 3,
        "startPeriod": 60
      }
    }
  ]
}
EOF

echo -e "${GREEN}✓ 任务定义已生成: task-definition.json${NC}"
echo ""

# 注册任务定义
echo -e "${YELLOW}  注册 ECS 任务定义...${NC}"
TASK_DEF_ARN=$(aws ecs register-task-definition \
    --region $AWS_REGION \
    --cli-input-json file://task-definition.json \
    --query 'taskDefinition.taskDefinitionArn' \
    --output text)
echo -e "${GREEN}✓ 任务定义注册成功: $TASK_DEF_ARN${NC}"
echo ""

# 显示部署命令
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}部署准备完成！${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo -e "${YELLOW}下一步：创建或更新 ECS 服务${NC}"
echo ""
echo "如果是首次部署，运行："
echo -e "${GREEN}aws ecs create-service \\${NC}"
echo -e "${GREEN}  --region $AWS_REGION \\${NC}"
echo -e "${GREEN}  --cluster $ECS_CLUSTER_NAME \\${NC}"
echo -e "${GREEN}  --service-name $SERVICE_NAME \\${NC}"
echo -e "${GREEN}  --task-definition $SERVICE_NAME \\${NC}"
echo -e "${GREEN}  --desired-count 1 \\${NC}"
echo -e "${GREEN}  --launch-type FARGATE \\${NC}"
echo -e "${GREEN}  --network-configuration \"awsvpcConfiguration={subnets=$SUBNETS,securityGroups=[$SECURITY_GROUP],assignPublicIp=${ECS_ASSIGN_PUBLIC_IP}}\"${NC}"
echo ""
echo "如果服务已存在，运行："
echo -e "${GREEN}aws ecs update-service \\${NC}"
echo -e "${GREEN}  --region $AWS_REGION \\${NC}"
echo -e "${GREEN}  --cluster $ECS_CLUSTER_NAME \\${NC}"
echo -e "${GREEN}  --service $SERVICE_NAME \\${NC}"
echo -e "${GREEN}  --task-definition $SERVICE_NAME${NC}"
echo ""
echo -e "${YELLOW}查看日志：${NC}"
echo -e "${GREEN}aws logs tail /ecs/$SERVICE_NAME --follow --region $AWS_REGION${NC}"
echo ""
echo -e "${YELLOW}验证部署：${NC}"
echo -e "${GREEN}# 获取任务公网 IP${NC}"
echo -e "${GREEN}aws ecs list-tasks --cluster $ECS_CLUSTER_NAME --service-name $SERVICE_NAME --region $AWS_REGION${NC}"
echo -e "${GREEN}# 然后访问 http://TASK_IP:8080/health${NC}"
echo ""

