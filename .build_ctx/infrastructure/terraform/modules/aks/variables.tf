# AKS Module Variables

variable "resource_group_name" {
  description = "Name of the resource group"
  type        = string
}

variable "location" {
  description = "Azure region for resources"
  type        = string
}

variable "prefix" {
  description = "Prefix for resource names"
  type        = string
}

variable "kubernetes_version" {
  description = "Kubernetes version"
  type        = string
  default     = "1.28"
}

variable "subnet_id" {
  description = "ID of the subnet for AKS nodes"
  type        = string
}

variable "system_node_count" {
  description = "Initial number of nodes in system pool"
  type        = number
  default     = 2
}

variable "system_node_size" {
  description = "VM size for system node pool"
  type        = string
  default     = "Standard_D4s_v3"
}

variable "user_node_count" {
  description = "Initial number of nodes in user pool"
  type        = number
  default     = 1
}

variable "user_node_size" {
  description = "VM size for user node pool"
  type        = string
  default     = "Standard_D8s_v3"
}

variable "log_analytics_workspace_id" {
  description = "ID of the Log Analytics workspace"
  type        = string
}

variable "acr_id" {
  description = "ID of the Azure Container Registry"
  type        = string
}

variable "tags" {
  description = "Tags to apply to all resources"
  type        = map(string)
  default     = {}
}

