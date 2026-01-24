# Key Vault Module Outputs

output "id" {
  description = "ID of the Key Vault"
  value       = azurerm_key_vault.main.id
}

output "name" {
  description = "Name of the Key Vault"
  value       = azurerm_key_vault.main.name
}

output "vault_uri" {
  description = "URI of the Key Vault"
  value       = azurerm_key_vault.main.vault_uri
}

output "redis_password" {
  description = "Generated Redis password"
  value       = random_password.redis.result
  sensitive   = true
}

output "jwt_secret" {
  description = "Generated JWT secret"
  value       = random_password.jwt_secret.result
  sensitive   = true
}

output "encryption_key" {
  description = "Generated encryption key"
  value       = random_password.encryption_key.result
  sensitive   = true
}

