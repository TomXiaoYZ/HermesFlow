# 创建IAM策略
resource "aws_iam_policy" "s3_access" {
  name        = "hermesflow-s3-access-${var.environment}"
  description = "Policy for accessing HermesFlow S3 bucket"

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Action = [
          "s3:ListBucket",
          "s3:GetBucketLocation"
        ]
        Resource = [aws_s3_bucket.hermesflow.arn]
      },
      {
        Effect = "Allow"
        Action = [
          "s3:PutObject",
          "s3:GetObject",
          "s3:DeleteObject"
        ]
        Resource = ["${aws_s3_bucket.hermesflow.arn}/*"]
      }
    ]
  })
}

# 将策略附加到EKS节点角色
resource "aws_iam_role_policy_attachment" "node_s3_access" {
  policy_arn = aws_iam_policy.s3_access.arn
  role       = var.eks_node_role_name
} 