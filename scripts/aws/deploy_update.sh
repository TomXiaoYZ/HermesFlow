#!/bin/bash
set -e

# Configuration
REGION="us-west-2"
ACCOUNT_ID=$(aws sts get-caller-identity --query Account --output text)
ECR_URL="${ACCOUNT_ID}.dkr.ecr.${REGION}.amazonaws.com"
APP_NAME="hermesflow"

# Get EC2 Public IP from AWS CLI
echo "🔍 Finding HermesFlow EC2 Instance..."
EC2_IP=$(aws ec2 describe-instances --region $REGION --filters "Name=tag:Name,Values=${APP_NAME}-node" "Name=instance-state-name,Values=running" --query "Reservations[0].Instances[0].PublicIpAddress" --output text)

if [ "$EC2_IP" == "None" ] || [ -z "$EC2_IP" ]; then
    echo "❌ Could not find running EC2 instance with tag Name=${APP_NAME}-node"
    echo "   Did you run scripts/aws/infra_init.sh?"
    exit 1
fi

echo "   Target Host: $EC2_IP"

# 1. Login to ECR
echo "------------------------------------------------"
echo "🔑 Logging in to ECR..."
aws ecr get-login-password --region $REGION | docker login --username AWS --password-stdin $ECR_URL

# 2. Build & Push Images
echo "------------------------------------------------"
echo "🏗️  Building and Pushing Docker Images..."

# Ensure build context is ready
./scripts/prepare_docker.sh

SERVICES=("data-engine" "strategy-engine" "execution-engine" "web")

for SERVICE in "${SERVICES[@]}"; do
    IMAGE_TAG="${ECR_URL}/${APP_NAME}-${SERVICE}:latest"
    echo "   Building $SERVICE..."
    
    # Enable platform flag for cross-compilation (Mac -> Linux/AMD64)
    # Note: This requires Docker Buildx
    docker buildx build --platform linux/amd64 \
        -t $IMAGE_TAG \
        -f services/$SERVICE/Dockerfile \
        --push \
        .build_ctx
        
    echo "   ✅ Pushed: $IMAGE_TAG"
done

# 3. Prepare Remote Config
echo "------------------------------------------------"
echo "📄 Preparing Remote Configuration..."

# Create dummy .env.prod if not exists locally, warn user to fill it
if [ ! -f .env.prod ]; then
    echo "⚠️  .env.prod not found locally! Creating a template."
    echo "   PLEASE EDIT .env.prod WITH REAL RDS CREDENTIALS BEFORE DEPLOYING."
    cat > .env.prod <<EOF
DATA_ENGINE__POSTGRES__HOST=fill_me_in_rds_endpoint
DATA_ENGINE__POSTGRES__PASSWORD=ChangeMe123!
DATA_ENGINE__POSTGRES__DATABASE=hermesflow
DATA_ENGINE__POSTGRES__USERNAME=postgres
EC2_PUBLIC_IP=$EC2_IP
EOF
    exit 1
fi

# Update EC2_IP in .env.prod dynamically
sed -i '' "s/^EC2_PUBLIC_IP=.*/EC2_PUBLIC_IP=$EC2_IP/" .env.prod || true

# 4. Deploy to EC2
echo "------------------------------------------------"
echo "🚀 Deploying to EC2 ($EC2_IP)..."
SSH_KEY="hermesflow-key.pem" # Assuming key is in current dir

if [ ! -f "$SSH_KEY" ]; then
    echo "❌ SSH Key ($SSH_KEY) not found in current directory."
    echo "   Please place the key file here to continue."
    exit 1
fi

# Copy config files
scp -o StrictHostKeyChecking=no -i $SSH_KEY docker-compose.prod.yml ubuntu@$EC2_IP:docker-compose.yml
scp -o StrictHostKeyChecking=no -i $SSH_KEY .env.prod ubuntu@$EC2_IP:.env.prod
# Copy clickhouse users.xml if needed (structure needs to exist remote, simplified for now)
# scp -r -i $SSH_KEY infrastructure ubuntu@$EC2_IP:infrastructure 

# Execute Remote Commands
ssh -o StrictHostKeyChecking=no -i $SSH_KEY ubuntu@$EC2_IP <<EOF
    # Login to ECR on remote
    aws ecr get-login-password --region $REGION | docker login --username AWS --password-stdin $ECR_URL
    
    # Export Registry URL for Compose
    export ECR_REGISTRY="${ECR_URL}/${APP_NAME}"
    
    # Pull latest images
    docker-compose pull
    
    # Restart services
    docker-compose up -d --remove-orphans
    
    # Cleanup old images
    docker image prune -f
EOF

echo "------------------------------------------------"
echo "✅ Deployment Complete!"
echo "   Dashboard: http://$EC2_IP"
