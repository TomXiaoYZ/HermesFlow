# ArgoCD Terraform模块 - 输出定义

output "namespace" {
  description = "ArgoCD部署的命名空间"
  value       = var.namespace
}

output "server_service_name" {
  description = "ArgoCD Server服务名称"
  value       = "argocd-server"
}

output "deployment_id" {
  description = "部署ID"
  value       = random_id.deployment_id.hex
}

output "admin_password" {
  description = "ArgoCD管理员密码"
  value       = var.admin_password
  sensitive   = true
}

output "server_endpoint" {
  description = "ArgoCD Server访问端点"
  value       = var.service_type == "LoadBalancer" ? "待LoadBalancer分配外部IP" : "使用kubectl port-forward访问"
}

output "access_instructions" {
  description = "ArgoCD访问说明"
  value = <<-EOF
    ArgoCD部署完成！
    
    访问方式：
    1. 如果使用LoadBalancer:
       - 等待外部IP分配: kubectl get svc -n argocd argocd-server
       - 通过浏览器访问: http://<EXTERNAL-IP>
    
    2. 如果使用端口转发:
       - 执行命令: kubectl port-forward svc/argocd-server -n argocd 8080:80
       - 通过浏览器访问: http://localhost:8080
    
    登录信息：
    - 用户名: admin
    - 密码: ${var.admin_password}
    
    常用命令：
    - 查看所有Pod状态: kubectl get pods -n argocd
    - 查看服务状态: kubectl get svc -n argocd
    - 查看ArgoCD日志: kubectl logs -n argocd -l app.kubernetes.io/name=argocd-server
  EOF
}

output "kubectl_commands" {
  description = "常用kubectl命令"
  value = {
    get_pods      = "kubectl get pods -n argocd"
    get_services  = "kubectl get svc -n argocd"
    port_forward  = "kubectl port-forward svc/argocd-server -n argocd 8080:80"
    get_external_ip = "kubectl get svc argocd-server -n argocd -o jsonpath='{.status.loadBalancer.ingress[0].ip}'"
    server_logs   = "kubectl logs -n argocd -l app.kubernetes.io/name=argocd-server"
  }
}

output "deployment_status" {
  description = "部署状态信息"
  value = {
    namespace_created = var.namespace
    components_deployed = "ArgoCD Server, Repository Server, Application Controller"
    rbac_configured = "是"
    admin_password_set = "是"
  }
}
