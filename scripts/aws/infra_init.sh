#!/bin/bash
set -e

# Configuration
APP_NAME="hermesflow"
AWS_REGION="us-west-2"
DB_USERNAME="postgres"
DB_PASSWORD="ChangeMe123!" # CHANGE THIS on production!
INSTANCE_TYPE="c5.xlarge"  # Balanced Hybrid choice

# Find latest Ubuntu 22.04 AMI for the region
echo "🔍 Finding latest Ubuntu 22.04 AMI in $AWS_REGION..."
AMI_ID=$(aws ec2 describe-images --region $AWS_REGION --owners 099720109477 --filters "Name=name,Values=ubuntu/images/hvm-ssd/ubuntu-jammy-22.04-amd64-server-*" "Name=state,Values=available" --query "Images | sort_by(@, &CreationDate) | [-1].ImageId" --output text)
echo "   Using AMI: $AMI_ID"

echo "🚀 Starting HermesFlow Infrastructure Initialization..."
echo "Region: $AWS_REGION"
echo "App Name: $APP_NAME"

# 0. Check AWS CLI
if ! command -v aws &> /dev/null; then
    echo "❌ AWS CLI could not be found. Please install it."
    exit 1
fi

# 1. Setup Networking (Security Group)
echo "------------------------------------------------"
echo "🌐 Setting up Security Group..."
VPC_ID=$(aws ec2 describe-vpcs --region $AWS_REGION --filters Name=isDefault,Values=true --query "Vpcs[0].VpcId" --output text)
echo "   Using Default VPC: $VPC_ID"

if ! aws ec2 describe-security-groups --region $AWS_REGION --group-names "${APP_NAME}-sg" &> /dev/null; then
    SG_ID=$(aws ec2 create-security-group --region $AWS_REGION --group-name "${APP_NAME}-sg" --description "Security group for HermesFlow" --vpc-id $VPC_ID --query "GroupId" --output text)
    echo "   Created Security Group: $SG_ID"
    
    # Allow SSH
    aws ec2 authorize-security-group-ingress --region $AWS_REGION --group-id $SG_ID --protocol tcp --port 22 --cidr 0.0.0.0/0
    # Allow HTTP/HTTPS
    aws ec2 authorize-security-group-ingress --region $AWS_REGION --group-id $SG_ID --protocol tcp --port 80 --cidr 0.0.0.0/0
    aws ec2 authorize-security-group-ingress --region $AWS_REGION --group-id $SG_ID --protocol tcp --port 443 --cidr 0.0.0.0/0
    # Allow Postgres from Self (RDS access)
    aws ec2 authorize-security-group-ingress --region $AWS_REGION --group-id $SG_ID --protocol tcp --port 5432 --source-group $SG_ID
    
    echo "   ✅ Security Group rules configured."
else
    SG_ID=$(aws ec2 describe-security-groups --region $AWS_REGION --group-names "${APP_NAME}-sg" --query "SecurityGroups[0].GroupId" --output text)
    echo "   ✅ Security Group exists: $SG_ID"
fi

# 2. IAM Role for EC2 (to pull from ECR)
echo "------------------------------------------------"
echo "🔑 Setting up IAM Role..."
ROLE_NAME="${APP_NAME}-ec2-role"
if ! aws iam get-role --role-name $ROLE_NAME &> /dev/null; then
    cat > trust-policy.json <<EOF
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Principal": { "Service": "ec2.amazonaws.com" },
      "Action": "sts:AssumeRole"
    }
  ]
}
EOF
    aws iam create-role --role-name $ROLE_NAME --assume-role-policy-document file://trust-policy.json > /dev/null
    rm trust-policy.json
    aws iam attach-role-policy --role-name $ROLE_NAME --policy-arn arn:aws:iam::aws:policy/AmazonEC2ContainerRegistryReadOnly
    aws iam create-instance-profile --instance-profile-name $ROLE_NAME > /dev/null
    aws iam add-role-to-instance-profile --instance-profile-name $ROLE_NAME --role-name $ROLE_NAME
    echo "   ✅ Created IAM Role & Instance Profile: $ROLE_NAME"
    # Wait for propagation
    sleep 5
else
    echo "   ✅ IAM Role exists: $ROLE_NAME"
fi

# 3. Create ECR Repositories
echo "------------------------------------------------"
echo "📦 Creating ECR Repositories..."
REPOS=("data-engine" "strategy-engine" "execution-engine" "web")
for REPO in "${REPOS[@]}"; do
    FULL_REPO_NAME="${APP_NAME}-${REPO}"
    if ! aws ecr describe-repositories --region $AWS_REGION --repository-names $FULL_REPO_NAME &> /dev/null; then
        aws ecr create-repository --region $AWS_REGION --repository-name $FULL_REPO_NAME > /dev/null
        echo "   Created: $FULL_REPO_NAME"
    else
        echo "   Exists: $FULL_REPO_NAME"
    fi
done

# 4. Provision RDS (Postgres)
echo "------------------------------------------------"
echo "🗄️  Provisioning RDS Postgres..."
DB_INSTANCE_ID="${APP_NAME}-rds"
if ! aws rds describe-db-instances --region $AWS_REGION --db-instance-identifier $DB_INSTANCE_ID &> /dev/null; then
    echo "   Creating RDS instance ($DB_INSTANCE_ID)... This will take a few minutes."
    aws rds create-db-instance \
        --region $AWS_REGION \
        --db-instance-identifier $DB_INSTANCE_ID \
        --db-instance-class db.t4g.micro \
        --engine postgres \
        --master-username $DB_USERNAME \
        --master-user-password $DB_PASSWORD \
        --allocated-storage 20 \
        --vpc-security-group-ids $SG_ID \
        --publicly-accessible \
        --backup-retention-period 7 \
        --no-cli-pager > /dev/null
    echo "   ⏳ RDS creation initiated."
else
    echo "   ✅ RDS instance exists: $DB_INSTANCE_ID"
fi

# 5. Provision EC2 Spot Instance
echo "------------------------------------------------"
echo "💻 Provisioning EC2 Spot Instance..."

# User Data to install Docker & Compose
cat > user-data.sh <<EOF
#!/bin/bash
apt-get update
apt-get install -y docker.io docker-compose
usermod -aG docker ubuntu
systemctl enable docker
systemctl start docker
EOF

if [ -z "$(aws ec2 describe-instances --region $AWS_REGION --filters "Name=tag:Name,Values=${APP_NAME}-node" "Name=instance-state-name,Values=running" --query "Reservations[].Instances[].InstanceId" --output text)" ]; then
    echo "   Requesting Spot Instance ($INSTANCE_TYPE)..."
    INSTANCE_ID=$(aws ec2 run-instances \
        --region $AWS_REGION \
        --image-id $AMI_ID \
        --count 1 \
        --instance-type $INSTANCE_TYPE \
        --key-name "hermesflow-key" \
        --security-group-ids $SG_ID \
        --iam-instance-profile Name=$ROLE_NAME \
        --instance-market-options '{"MarketType":"spot"}' \
        --tag-specifications "ResourceType=instance,Tags=[{Key=Name,Value=${APP_NAME}-node}]" \
        --user-data file://user-data.sh \
        --query "Instances[0].InstanceId" \
        --output text)
    
    echo "   ✅ Launched EC2 Instance: $INSTANCE_ID"
    rm user-data.sh
else
    echo "   ✅ Running EC2 instance found."
    # Get Public IP
    PUBLIC_IP=$(aws ec2 describe-instances --region $AWS_REGION --filters "Name=tag:Name,Values=${APP_NAME}-node" "Name=instance-state-name,Values=running" --query "Reservations[0].Instances[0].PublicIpAddress" --output text)
    echo "   Public IP: $PUBLIC_IP"
fi

echo "------------------------------------------------"
echo "🎉 Infrastructure setup complete/initiated!"
echo "NOTE: RDS creation happens in background. Check AWS Console for status."
echo "NEXT: Run scripts/aws/deploy_update.sh to deploy code."
