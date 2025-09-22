# HermesFlow Dev环境变量

variable "location" {
  description = "Azure部署区域"
  type        = string
  default     = "East US"
  
  validation {
    condition = contains([
      "East US", "East US 2", "West US", "West US 2", "West US 3",
      "Central US", "North Central US", "South Central US", "West Central US"
    ], var.location)
    error_message = "Location必须是支持的Azure区域。"
  }
}

variable "enable_monitoring" {
  description = "是否启用增强监控"
  type        = bool
  default     = false  # Dev环境可以关闭以节省成本
}

variable "enable_backup" {
  description = "是否启用备份"
  type        = bool
  default     = false  # Dev环境可以关闭备份
}

variable "argocd_admin_password" {
  description = "ArgoCD管理员密码（明文）"
  type        = string
  default     = "admin123"
  sensitive   = true
}
