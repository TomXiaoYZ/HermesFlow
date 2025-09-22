#!/bin/bash
# HermesFlow 统一模块构建脚本

# 从环境变量获取配置
MODULE="$MODULE"
TAG="${GITHUB_SHA:-latest}"
ENVIRONMENT="$ENVIRONMENT"
REGISTRY="$AZURE_REGISTRY"

echo "🏗️ HermesFlow Module Build Configuration:"
echo "  Module: $MODULE"
echo "  Tag: $TAG"
echo "  Environment: $ENVIRONMENT"
echo "  Registry: $REGISTRY"

# 验证必要的环境变量
if [[ -z "$MODULE" || -z "$REGISTRY" || -z "$AZURE_CLIENT_ID" || -z "$AZURE_CLIENT_SECRET" ]]; then
    echo "❌ Missing required environment variables"
    echo "Required: MODULE, REGISTRY, AZURE_CLIENT_ID, AZURE_CLIENT_SECRET"
    exit 1
fi

# 检查模块目录和Dockerfile
if [[ ! -d "modules/$MODULE" ]]; then
    echo "❌ Module directory not found: modules/$MODULE"
    exit 1
fi

if [[ ! -f "scripts/$MODULE/Dockerfile" ]]; then
    echo "❌ Dockerfile not found: scripts/$MODULE/Dockerfile"
    exit 1
fi

# 进入模块目录进行构建
cd "modules/$MODULE"

echo "🔨 Building module source code..."

# 根据模块类型进行构建
case "$MODULE" in
    strategy-engine|risk-engine|user-management|api-gateway)
        echo "☕ Building Java module..."
        if [[ -f "pom.xml" ]]; then
            mvn clean package -DskipTests -B
            if [[ $? -ne 0 ]]; then
                echo "❌ Maven build failed"
                exit 1
            fi
        else
            echo "⚠️ No pom.xml found, skipping Maven build"
        fi
        ;;
    data-engine)
        echo "🐍 Building Python module..."
        if [[ -f "requirements.txt" ]]; then
            pip install -r requirements.txt
            if [[ $? -ne 0 ]]; then
                echo "❌ pip install failed"
                exit 1
            fi
        else
            echo "⚠️ No requirements.txt found, skipping pip install"
        fi
        ;;
    frontend)
        echo "📦 Building Node.js module..."
        if [[ -f "package.json" ]]; then
            npm ci && npm run build
            if [[ $? -ne 0 ]]; then
                echo "❌ npm build failed"
                exit 1
            fi
        else
            echo "⚠️ No package.json found, skipping npm build"
        fi
        ;;
    *)
        echo "⚠️ Unknown module type: $MODULE, skipping build step"
        ;;
esac

# 返回根目录
cd ../../

# 构建Docker镜像
IMAGE_NAME="$REGISTRY/$MODULE"
IMAGE_TAG="$IMAGE_NAME:$TAG"
LATEST_TAG="$IMAGE_NAME:$ENVIRONMENT-latest"

echo "🐳 Building Docker image..."
echo "  Full tag: $IMAGE_TAG"
echo "  Latest tag: $LATEST_TAG"

docker build \
    -t "$IMAGE_TAG" \
    -t "$LATEST_TAG" \
    -f "scripts/$MODULE/Dockerfile" \
    "modules/$MODULE"

if [[ $? -ne 0 ]]; then
    echo "❌ Docker build failed"
    exit 1
fi

# 登录Azure Container Registry
echo "🔐 Logging into Azure Container Registry..."
echo "$AZURE_CLIENT_SECRET" | docker login "$REGISTRY" \
    -u "$AZURE_CLIENT_ID" --password-stdin

if [[ $? -ne 0 ]]; then
    echo "❌ Failed to login to Azure Container Registry"
    exit 1
fi

# 推送镜像
echo "📤 Pushing images to registry..."
docker push "$IMAGE_TAG"
docker push "$LATEST_TAG"

if [[ $? -eq 0 ]]; then
    echo "✅ Successfully built and pushed $MODULE"
    echo "  Image: $IMAGE_TAG"
    echo "  Latest: $LATEST_TAG"
else
    echo "❌ Failed to push images"
    exit 1
fi

