# ArgoCD Terraform模块 - 变量定义

variable "admin_password" {
  description = "ArgoCD管理员密码"
  type        = string
  default     = "admin123"
  sensitive   = true
}

variable "argocd_version" {
  description = "ArgoCD镜像版本"
  type        = string
  default     = "v2.8.0"
}

variable "argocd_chart_version" {
  description = "ArgoCD Helm Chart版本"
  type        = string
  default     = "5.51.6"
}

variable "wait_for_ready" {
  description = "是否等待ArgoCD就绪"
  type        = bool
  default     = true
}

variable "enable_high_availability" {
  description = "是否启用高可用模式"
  type        = bool
  default     = false
}

variable "server_replicas" {
  description = "ArgoCD Server副本数"
  type        = number
  default     = 1
}

variable "repo_server_replicas" {
  description = "ArgoCD Repository Server副本数"
  type        = number
  default     = 1
}

variable "controller_replicas" {
  description = "ArgoCD Application Controller副本数"
  type        = number
  default     = 1
}

variable "service_type" {
  description = "ArgoCD Server服务类型 (ClusterIP, NodePort, LoadBalancer)"
  type        = string
  default     = "LoadBalancer"
  
  validation {
    condition     = contains(["ClusterIP", "NodePort", "LoadBalancer"], var.service_type)
    error_message = "服务类型必须是 ClusterIP, NodePort 或 LoadBalancer 之一."
  }
}

variable "enable_insecure" {
  description = "是否启用不安全模式 (HTTP)"
  type        = bool
  default     = true
}

variable "namespace" {
  description = "ArgoCD命名空间"
  type        = string
  default     = "argocd"
}

variable "resource_limits" {
  description = "资源限制配置"
  type = object({
    server = object({
      cpu    = string
      memory = string
    })
    repo_server = object({
      cpu    = string
      memory = string
    })
    controller = object({
      cpu    = string
      memory = string
    })
  })
  default = {
    server = {
      cpu    = "500m"
      memory = "512Mi"
    }
    repo_server = {
      cpu    = "500m"
      memory = "512Mi"
    }
    controller = {
      cpu    = "1000m"
      memory = "1Gi"
    }
  }
}

variable "resource_requests" {
  description = "资源请求配置"
  type = object({
    server = object({
      cpu    = string
      memory = string
    })
    repo_server = object({
      cpu    = string
      memory = string
    })
    controller = object({
      cpu    = string
      memory = string
    })
  })
  default = {
    server = {
      cpu    = "100m"
      memory = "128Mi"
    }
    repo_server = {
      cpu    = "100m"
      memory = "128Mi"
    }
    controller = {
      cpu    = "200m"
      memory = "256Mi"
    }
  }
}

variable "tags" {
  description = "资源标签"
  type        = map(string)
  default     = {}
}
