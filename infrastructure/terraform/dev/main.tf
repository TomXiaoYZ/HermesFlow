terraform {
  required_version = ">= 1.0"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "~> 2.0"
    }
  }

  backend "s3" {
    bucket = "hermesflow-terraform-state"
    key    = "dev/terraform.tfstate"
    region = "us-west-2"
  }
}

provider "aws" {
  region = var.aws_region

  default_tags {
    tags = {
      Environment = "development"
      Project     = "hermesflow"
      ManagedBy   = "terraform"
    }
  }
}

module "vpc" {
  source = "../modules/vpc"

  environment = "dev"
  vpc_cidr    = "10.0.0.0/16"
  azs         = ["us-west-2a", "us-west-2b", "us-west-2c"]
}

module "eks" {
  source = "../modules/eks"

  environment         = "dev"
  cluster_name        = "hermesflow-dev"
  vpc_id             = module.vpc.vpc_id
  private_subnet_ids = module.vpc.private_subnet_ids

  node_groups = {
    general = {
      desired_size = 2
      min_size     = 1
      max_size     = 3

      instance_types = ["t3.large"]
      capacity_type  = "ON_DEMAND"

      labels = {
        role = "general"
      }

      taints = []
    }

    monitoring = {
      desired_size = 1
      min_size     = 1
      max_size     = 2

      instance_types = ["t3.large"]
      capacity_type  = "ON_DEMAND"

      labels = {
        role = "monitoring"
      }

      taints = []
    }
  }
}

module "s3" {
  source = "../modules/s3"

  environment = "dev"
  bucket_name = "hermesflow-dev"
} 