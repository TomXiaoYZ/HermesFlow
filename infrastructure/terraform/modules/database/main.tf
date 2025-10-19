# Database Module - PostgreSQL Flexible Server
# Managed PostgreSQL database for HermesFlow application data

resource "azurerm_private_dns_zone" "postgres" {
  name                = "${var.prefix}.postgres.database.azure.com"
  resource_group_name = var.resource_group_name
  tags                = var.tags
}

resource "azurerm_private_dns_zone_virtual_network_link" "postgres" {
  name                  = "${var.prefix}-postgres-vnet-link"
  private_dns_zone_name = azurerm_private_dns_zone.postgres.name
  resource_group_name   = var.resource_group_name
  virtual_network_id    = var.vnet_id
  tags                  = var.tags
}

resource "azurerm_postgresql_flexible_server" "main" {
  name                   = "${var.prefix}-postgres"
  resource_group_name    = var.resource_group_name
  location               = var.location
  version                = "15"
  delegated_subnet_id    = var.subnet_id
  private_dns_zone_id    = azurerm_private_dns_zone.postgres.id
  administrator_login    = var.admin_username
  administrator_password = var.admin_password
  zone                   = "1"
  storage_mb             = var.storage_mb
  sku_name               = var.sku_name
  backup_retention_days  = var.backup_retention_days
  
  # Disable public access when using VNet integration
  public_network_access_enabled = false

  dynamic "high_availability" {
    for_each = var.environment == "production" ? [1] : []
    content {
      mode                      = "ZoneRedundant"
      standby_availability_zone = "2"
    }
  }

  maintenance_window {
    day_of_week  = 0 # Sunday
    start_hour   = 3
    start_minute = 0
  }

  tags = var.tags

  depends_on = [azurerm_private_dns_zone_virtual_network_link.postgres]
}

# Create hermesflow database
resource "azurerm_postgresql_flexible_server_database" "hermesflow" {
  name      = "hermesflow"
  server_id = azurerm_postgresql_flexible_server.main.id
  collation = "en_US.utf8"
  charset   = "utf8"
}

# Firewall rule to allow AKS access (only for public network access)
# Not needed when using VNet integration
resource "azurerm_postgresql_flexible_server_firewall_rule" "aks" {
  count            = var.aks_outbound_ip != "" && var.aks_outbound_ip != null ? 1 : 0
  name             = "AllowAKS"
  server_id        = azurerm_postgresql_flexible_server.main.id
  start_ip_address = var.aks_outbound_ip
  end_ip_address   = var.aks_outbound_ip
}

# PostgreSQL configurations for performance
resource "azurerm_postgresql_flexible_server_configuration" "max_connections" {
  name      = "max_connections"
  server_id = azurerm_postgresql_flexible_server.main.id
  value     = var.environment == "production" ? "200" : "100"
}

resource "azurerm_postgresql_flexible_server_configuration" "shared_buffers" {
  name      = "shared_buffers"
  server_id = azurerm_postgresql_flexible_server.main.id
  value     = var.environment == "production" ? "524288" : "262144" # 2GB : 1GB (in 8KB pages)
}

