# Monitoring Module - Log Analytics and Alerts
# Centralized monitoring and alerting for HermesFlow infrastructure

resource "azurerm_log_analytics_workspace" "main" {
  name                = "${var.prefix}-logs"
  location            = var.location
  resource_group_name = var.resource_group_name
  sku                 = "PerGB2018"
  retention_in_days   = var.log_retention_days
  tags                = var.tags
}

resource "azurerm_log_analytics_solution" "container_insights" {
  solution_name         = "ContainerInsights"
  location              = var.location
  resource_group_name   = var.resource_group_name
  workspace_resource_id = azurerm_log_analytics_workspace.main.id
  workspace_name        = azurerm_log_analytics_workspace.main.name

  plan {
    publisher = "Microsoft"
    product   = "OMSGallery/ContainerInsights"
  }

  tags = var.tags
}

# Saved query for Pod errors
resource "azurerm_log_analytics_saved_search" "pod_errors" {
  name                       = "PodErrors"
  log_analytics_workspace_id = azurerm_log_analytics_workspace.main.id
  category                   = "Kubernetes"
  display_name               = "Pod Errors"
  query                      = <<-QUERY
    KubePodInventory
    | where ClusterName == "${var.cluster_name}"
    | where PodStatus == "Failed" or PodStatus == "CrashLoopBackOff"
    | project TimeGenerated, Namespace, Name, PodStatus, ContainerStatusReason
    | order by TimeGenerated desc
  QUERY
}

# Saved query for high CPU usage
resource "azurerm_log_analytics_saved_search" "high_cpu" {
  name                       = "HighCPUUsage"
  log_analytics_workspace_id = azurerm_log_analytics_workspace.main.id
  category                   = "Performance"
  display_name               = "High CPU Usage"
  query                      = <<-QUERY
    Perf
    | where ObjectName == "Processor" and CounterName == "% Processor Time"
    | where CounterValue > 80
    | summarize avg(CounterValue) by Computer, bin(TimeGenerated, 5m)
    | order by TimeGenerated desc
  QUERY
}

# Action Group for alerts
resource "azurerm_monitor_action_group" "main" {
  name                = "${var.prefix}-action-group"
  resource_group_name = var.resource_group_name
  short_name          = "HermesOps"

  email_receiver {
    name                    = "DevOps Team"
    email_address           = var.alert_email
    use_common_alert_schema = true
  }

  tags = var.tags
}

# Metric Alert for AKS CPU usage
resource "azurerm_monitor_metric_alert" "aks_cpu" {
  count               = var.aks_id != "" ? 1 : 0
  name                = "${var.prefix}-aks-cpu-alert"
  resource_group_name = var.resource_group_name
  scopes              = [var.aks_id]
  description         = "Alert when AKS cluster CPU usage is high"
  severity            = 2
  frequency           = "PT5M"
  window_size         = "PT15M"
  enabled             = true

  criteria {
    metric_namespace = "Microsoft.ContainerService/managedClusters"
    metric_name      = "node_cpu_usage_percentage"
    aggregation      = "Average"
    operator         = "GreaterThan"
    threshold        = 80
  }

  action {
    action_group_id = azurerm_monitor_action_group.main.id
  }

  tags = var.tags
}

# Metric Alert for AKS Memory usage
resource "azurerm_monitor_metric_alert" "aks_memory" {
  count               = var.aks_id != "" ? 1 : 0
  name                = "${var.prefix}-aks-memory-alert"
  resource_group_name = var.resource_group_name
  scopes              = [var.aks_id]
  description         = "Alert when AKS cluster memory usage is high"
  severity            = 2
  frequency           = "PT5M"
  window_size         = "PT15M"
  enabled             = true

  criteria {
    metric_namespace = "Microsoft.ContainerService/managedClusters"
    metric_name      = "node_memory_working_set_percentage"
    aggregation      = "Average"
    operator         = "GreaterThan"
    threshold        = 85
  }

  action {
    action_group_id = azurerm_monitor_action_group.main.id
  }

  tags = var.tags
}

# Metric Alert for AKS Pod count
resource "azurerm_monitor_metric_alert" "aks_pod_count" {
  count               = var.aks_id != "" ? 1 : 0
  name                = "${var.prefix}-aks-pod-count-alert"
  resource_group_name = var.resource_group_name
  scopes              = [var.aks_id]
  description         = "Alert when AKS pod count is near limit"
  severity            = 3
  frequency           = "PT5M"
  window_size         = "PT15M"
  enabled             = true

  criteria {
    metric_namespace = "Microsoft.ContainerService/managedClusters"
    metric_name      = "kube_pod_status_ready"
    aggregation      = "Average"
    operator         = "LessThan"
    threshold        = 0.8
  }

  action {
    action_group_id = azurerm_monitor_action_group.main.id
  }

  tags = var.tags
}

