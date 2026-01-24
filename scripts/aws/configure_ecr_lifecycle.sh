#!/bin/bash
set -e

REGION="us-west-2"
APP_NAME="hermesflow"
REPOS=("data-engine" "strategy-engine" "execution-engine" "web")

# Define Policy: Keep only last 5 images
cat > lifecycle-policy.json <<EOF
{
    "rules": [
        {
            "rulePriority": 1,
            "description": "Keep last 5 images",
            "selection": {
                "tagStatus": "any",
                "countType": "imageCountMoreThan",
                "countNumber": 5
            },
            "action": {
                "type": "expire"
            }
        }
    ]
}
EOF

echo "🧹 Configuring ECR Lifecycle Policies (Keep last 5 images)..."

for REPO in "${REPOS[@]}"; do
    FULL_REPO_NAME="${APP_NAME}-${REPO}"
    echo "   Setting policy for $FULL_REPO_NAME..."
    aws ecr put-lifecycle-policy \
        --region $REGION \
        --repository-name $FULL_REPO_NAME \
        --lifecycle-policy-text file://lifecycle-policy.json > /dev/null
done

rm lifecycle-policy.json
echo "✅ ECR Lifecycle Configured."
