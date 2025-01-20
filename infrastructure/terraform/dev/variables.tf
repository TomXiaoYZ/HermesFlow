variable "aws_region" {
  description = "AWS region"
  type        = string
  default     = "us-west-2"
}

variable "environment" {
  description = "Environment name"
  type        = string
  default     = "dev"
}

variable "project" {
  description = "Project name"
  type        = string
  default     = "hermesflow"
}

variable "vpc_cidr" {
  description = "VPC CIDR block"
  type        = string
  default     = "10.0.0.0/16"
}

variable "availability_zones" {
  description = "List of availability zones"
  type        = list(string)
  default     = ["us-west-2a", "us-west-2b", "us-west-2c"]
}

variable "cluster_version" {
  description = "Kubernetes version"
  type        = string
  default     = "1.27"
}

variable "node_instance_types" {
  description = "List of instance types for the node groups"
  type        = map(list(string))
  default = {
    general = ["t3.large"]
    monitoring = ["t3.large"]
  }
}

variable "node_group_scaling" {
  description = "Node group autoscaling configuration"
  type = map(object({
    min_size     = number
    max_size     = number
    desired_size = number
  }))
  default = {
    general = {
      min_size     = 1
      max_size     = 3
      desired_size = 2
    }
    monitoring = {
      min_size     = 1
      max_size     = 2
      desired_size = 1
    }
  }
} 