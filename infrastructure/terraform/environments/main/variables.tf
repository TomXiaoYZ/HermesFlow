# HermesFlow Main环境变量

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
  default     = true  # 生产环境启用监控
}

variable "enable_backup" {
  description = "是否启用备份"
  type        = bool
  default     = true  # 生产环境启用备份
}

variable "enable_workload_separation" {
  description = "是否启用工作负载分离(独立节点池)"
  type        = bool
  default     = false  # 可选启用，适合大规模部署
}

variable "high_availability" {
  description = "是否启用高可用配置"
  type        = bool
  default     = true
}
