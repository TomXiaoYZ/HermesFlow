#!/bin/bash
set -e

# Usage: ./deploy.sh [app|gateway|all]
DEPLOY_MODE=${1:-app}

# Configuration
AWS_REGION="us-west-2"
AWS_ACCOUNT_ID=$(aws sts get-caller-identity --query Account --output text)
ECR_REPO="data-engine"
ECR_URI="${AWS_ACCOUNT_ID}.dkr.ecr.${AWS_REGION}.amazonaws.com"
CLUSTER_NAME="DifyOnAwsStack-ClusterEB0386A7-d4ySDF3hYoIw"
DATA_ENGINE_SERVICE="data-engine-service"
IB_GATEWAY_SERVICE="ib-gateway-service"
IAM_ROLE_ARN="arn:aws:iam::${AWS_ACCOUNT_ID}:role/ecsTaskExecutionRole"

echo "Deploying to AWS Account: $AWS_ACCOUNT_ID Region: $AWS_REGION Mode: $DEPLOY_MODE"

# Function to deploy Data Engine
deploy_app() {
    echo "=== Deploying Data Engine ==="
    
    # login (Skipped as per previous script, uncomment if needed)
    # aws ecr get-login-password --region $AWS_REGION | docker login --username AWS --password-stdin $ECR_URI

    echo "Building Docker image..."
    
    # Copy migrations to build context
    echo "Copying migrations..."
    rm -rf modules/data-engine/migrations
    mkdir -p modules/data-engine/migrations
    cp scripts/db/*.sql modules/data-engine/migrations/

    cd modules/data-engine
    docker build --platform linux/amd64 -t $ECR_REPO:latest .
    
    # Cleanup
    rm -rf migrations
    cd ../..
    docker tag $ECR_REPO:latest $ECR_URI/$ECR_REPO:latest

    echo "Pushing to ECR..."
    docker push $ECR_URI/$ECR_REPO:latest

    echo "Preparing Task Definition..."
    sed -e "s|\${ECR_URI}|${ECR_URI}|g" \
        -e "s|\${ECS_EXECUTION_ROLE_ARN}|${IAM_ROLE_ARN}|g" \
        -e "s|\${ECS_TASK_ROLE_ARN}|${IAM_ROLE_ARN}|g" \
        -e "s|\${IB_USER}|${IB_USER}|g" \
        -e "s|\${IB_PASS}|${IB_PASS}|g" \
        infrastructure/aws/ecs/sidecar-task-definition.json > task-def-app.json

    echo "Registering Task Definition..."
    REVISION=$(aws ecs register-task-definition --cli-input-json file://task-def-app.json --region $AWS_REGION --query 'taskDefinition.revision' --output text)
    echo "Registered Data Engine (Sidecar) Revision: $REVISION"
    rm task-def-app.json

    echo "Updating ECS Service ($DATA_ENGINE_SERVICE)..."
    aws ecs update-service \
        --cluster $CLUSTER_NAME \
        --service $DATA_ENGINE_SERVICE \
        --task-definition data-engine-task:$REVISION \
        --network-configuration "awsvpcConfiguration={subnets=[subnet-03b23e5b84f561f3a,subnet-0b6fb7c95fc8c2236],securityGroups=[sg-0eb44e4529b492928],assignPublicIp=ENABLED}" \
        --force-new-deployment \
        --region $AWS_REGION
}

# Function to deploy IB Gateway
deploy_gateway() {
    echo "=== Deploying IB Gateway ==="
    
    echo "Preparing Task Definition..."
    sed -e "s|\${ECS_EXECUTION_ROLE_ARN}|${IAM_ROLE_ARN}|g" \
        -e "s|\${ECS_TASK_ROLE_ARN}|${IAM_ROLE_ARN}|g" \
        -e "s|\${IB_USER}|${IB_USER}|g" \
        -e "s|\${IB_PASS}|${IB_PASS}|g" \
        infrastructure/aws/ecs/ib-gateway-task-definition.json > task-def-gateway.json
        
    echo "Registering Task Definition..."
    REVISION=$(aws ecs register-task-definition --cli-input-json file://task-def-gateway.json --region $AWS_REGION --query 'taskDefinition.revision' --output text)
    echo "Registered IB Gateway Revision: $REVISION"
    rm task-def-gateway.json
    
    echo "Updating ECS Service ($IB_GATEWAY_SERVICE)..."
    # Note: Requires service to be created first with Service Discovery enabled
    aws ecs update-service \
        --cluster $CLUSTER_NAME \
        --service $IB_GATEWAY_SERVICE \
        --task-definition ib-gateway-task:$REVISION \
        --network-configuration "awsvpcConfiguration={subnets=[subnet-03b23e5b84f561f3a,subnet-0b6fb7c95fc8c2236],securityGroups=[sg-0eb44e4529b492928],assignPublicIp=ENABLED}" \
        --region $AWS_REGION
}

# Execution Logic
if [[ "$DEPLOY_MODE" == "app" || "$DEPLOY_MODE" == "all" ]]; then
    deploy_app
fi

if [[ "$DEPLOY_MODE" == "gateway" || "$DEPLOY_MODE" == "all" ]]; then
    deploy_gateway
fi

echo "Deployment completed successfully!"
echo "Monitor: aws logs tail /ecs/data-engine --follow --region $AWS_REGION"

