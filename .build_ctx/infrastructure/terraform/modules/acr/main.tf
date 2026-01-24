# ACR Module - Azure Container Registry
# Manages Docker container images for HermesFlow microservices

resource "azurerm_container_registry" "main" {
  name                = replace("${var.prefix}acr", "-", "") # ACR names can't contain hyphens
  resource_group_name = var.resource_group_name
  location            = var.location
  sku                 = var.sku
  admin_enabled       = false # Use Managed Identity instead

  # Geo-replication for production environments
  dynamic "georeplications" {
    for_each = var.georeplications
    content {
      location                = georeplications.value.location
      zone_redundancy_enabled = georeplications.value.zone_redundancy_enabled
      tags                    = var.tags
    }
  }

  # Network rules - only for Premium SKU
  dynamic "network_rule_set" {
    for_each = var.sku == "Premium" ? [1] : []
    content {
      default_action = var.environment == "production" ? "Deny" : "Allow"
    }
  }

  tags = var.tags
}

# Diagnostic settings for monitoring
resource "azurerm_monitor_diagnostic_setting" "acr" {
  name                       = "${var.prefix}-acr-diag"
  target_resource_id         = azurerm_container_registry.main.id
  log_analytics_workspace_id = var.log_analytics_workspace_id

  enabled_log {
    category = "ContainerRegistryRepositoryEvents"
  }

  enabled_log {
    category = "ContainerRegistryLoginEvents"
  }

  metric {
    category = "AllMetrics"
    enabled  = true
  }
}

