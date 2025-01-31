terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
}

# S3存储桶配置
resource "aws_s3_bucket" "hermesflow" {
  bucket = "hermesflow-${var.environment}"

  tags = {
    Name        = "hermesflow-${var.environment}"
    Environment = var.environment
    ManagedBy   = "terraform"
  }
}

# 版本控制配置
resource "aws_s3_bucket_versioning" "hermesflow" {
  bucket = aws_s3_bucket.hermesflow.id
  versioning_configuration {
    status = "Enabled"
  }
}

# 服务器端加密配置
resource "aws_s3_bucket_server_side_encryption_configuration" "hermesflow" {
  bucket = aws_s3_bucket.hermesflow.id

  rule {
    apply_server_side_encryption_by_default {
      sse_algorithm = "AES256"
    }
  }
}

# 生命周期规则配置
resource "aws_s3_bucket_lifecycle_configuration" "hermesflow" {
  bucket = aws_s3_bucket.hermesflow.id

  rule {
    id     = "logs"
    status = "Enabled"

    filter {
      prefix = "logs/"
    }

    transition {
      days          = 30
      storage_class = "STANDARD_IA"
    }

    transition {
      days          = 60
      storage_class = "GLACIER"
    }

    expiration {
      days = 90
    }
  }

  rule {
    id     = "market-data"
    status = "Enabled"

    filter {
      prefix = "market-data/"
    }

    transition {
      days          = 30
      storage_class = "STANDARD_IA"
    }
  }
}

# CORS配置
resource "aws_s3_bucket_cors_configuration" "hermesflow" {
  bucket = aws_s3_bucket.hermesflow.id

  cors_rule {
    allowed_headers = ["*"]
    allowed_methods = ["GET", "PUT", "POST", "DELETE"]
    allowed_origins = ["*"]  # 生产环境需要限制具体域名
    expose_headers  = ["ETag"]
    max_age_seconds = 3000
  }
}

# 公共访问阻止配置
resource "aws_s3_bucket_public_access_block" "hermesflow" {
  bucket = aws_s3_bucket.hermesflow.id

  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
} 