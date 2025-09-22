# HermesFlow AKS 模块变量

variable "cluster_name" {
  description = "AKS集群名称"
  type        = string
}

variable "resource_group_name" {
  description = "资源组名称"
  type        = string
}

variable "location" {
  description = "Azure区域"
  type        = string
  default     = "East US"
}

variable "dns_prefix" {
  description = "DNS前缀"
  type        = string
}

variable "kubernetes_version" {
  description = "Kubernetes版本"
  type        = string
  default     = "1.28.3"
}

# 节点配置
variable "node_count" {
  description = "默认节点数量"
  type        = number
  default     = 2
}

variable "vm_size" {
  description = "虚拟机规格"
  type        = string
  default     = "Standard_B2s"  # 成本优化规格
}

variable "os_disk_size_gb" {
  description = "OS磁盘大小 (GB)"
  type        = number
  default     = 30
}

variable "os_disk_type" {
  description = "OS磁盘类型"
  type        = string
  default     = "Managed"
}

# 自动扩缩容配置
variable "enable_auto_scaling" {
  description = "是否启用自动扩缩容"
  type        = bool
  default     = true
}

variable "min_node_count" {
  description = "最小节点数量"
  type        = number
  default     = 1
}

variable "max_node_count" {
  description = "最大节点数量"
  type        = number
  default     = 5
}

# 网络配置
variable "subnet_id" {
  description = "子网ID"
  type        = string
  default     = null
}

variable "dns_service_ip" {
  description = "DNS服务IP"
  type        = string
  default     = "10.240.0.10"
}

variable "service_cidr" {
  description = "服务CIDR"
  type        = string
  default     = "10.240.0.0/16"
}

# 监控配置
variable "log_retention_days" {
  description = "日志保留天数"
  type        = number
  default     = 30
}

# ACR集成
variable "acr_id" {
  description = "Azure Container Registry ID"
  type        = string
  default     = null
}

# 工作负载节点池配置
variable "enable_workload_node_pool" {
  description = "是否启用工作负载节点池"
  type        = bool
  default     = false
}

variable "workload_vm_size" {
  description = "工作负载节点VM规格"
  type        = string
  default     = "Standard_D2s_v3"
}

variable "workload_node_count" {
  description = "工作负载节点数量"
  type        = number
  default     = 1
}

variable "workload_min_count" {
  description = "工作负载节点最小数量"
  type        = number
  default     = 1
}

variable "workload_max_count" {
  description = "工作负载节点最大数量"
  type        = number
  default     = 3
}

variable "workload_node_taints" {
  description = "工作负载节点污点"
  type        = list(string)
  default     = ["workload=true:NoSchedule"]
}

variable "workload_node_labels" {
  description = "工作负载节点标签"
  type        = map(string)
  default = {
    "node-type" = "workload"
  }
}

variable "tags" {
  description = "资源标签"
  type        = map(string)
  default = {
    Environment = "dev"
    Project     = "HermesFlow"
    ManagedBy   = "Terraform"
  }
}
