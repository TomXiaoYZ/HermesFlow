locals {
  name = var.cluster_name
}

module "eks" {
  source  = "terraform-aws-modules/eks/aws"
  version = "~> 19.0"

  cluster_name                   = local.name
  cluster_version               = var.cluster_version
  cluster_endpoint_public_access = true

  vpc_id     = var.vpc_id
  subnet_ids = var.private_subnet_ids

  eks_managed_node_groups = {
    for name, config in var.node_groups : name => {
      name = "${local.name}-${name}"

      min_size     = config.min_size
      max_size     = config.max_size
      desired_size = config.desired_size

      instance_types = config.instance_types
      capacity_type  = config.capacity_type

      labels = merge(
        {
          "node-group" = name
        },
        config.labels
      )

      taints = config.taints
    }
  }

  # aws-auth configmap
  manage_aws_auth_configmap = true

  aws_auth_roles = [
    {
      rolearn  = "arn:aws:iam::${data.aws_caller_identity.current.account_id}:role/admin"
      username = "admin"
      groups   = ["system:masters"]
    },
  ]

  tags = {
    Environment = var.environment
    Project     = "hermesflow"
    ManagedBy   = "terraform"
  }
}

# Get current AWS account ID
data "aws_caller_identity" "current" {} 