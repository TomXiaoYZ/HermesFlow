# ArgoCD Terraform模块 - 使用local-exec部署
terraform {
  required_providers {
    null = {
      source  = "hashicorp/null"
      version = "~> 3.0"
    }
    random = {
      source  = "hashicorp/random"
      version = "~> 3.0"
    }
  }
}

# 创建ArgoCD命名空间
resource "null_resource" "create_namespace" {
  provisioner "local-exec" {
    command = <<-EOT
      kubectl create namespace ${var.namespace} --dry-run=client -o yaml | kubectl apply -f -
    EOT
  }

  provisioner "local-exec" {
    when    = destroy
    command = <<-EOT
      kubectl delete namespace argocd --ignore-not-found=true
    EOT
  }
}

# 准备ArgoCD安装清单 (使用本地缓存避免网络问题)
resource "null_resource" "prepare_argocd_manifest" {
  provisioner "local-exec" {
    command = <<-EOT
      mkdir -p /tmp/argocd
      if [ ! -f /tmp/argocd/install.yaml ]; then
        cp /tmp/argocd-install.yaml /tmp/argocd/install.yaml 2>/dev/null || \
        curl -sSL -o /tmp/argocd/install.yaml https://raw.githubusercontent.com/argoproj/argo-cd/stable/manifests/install.yaml --connect-timeout 30 || \
        echo "警告: 无法下载ArgoCD清单，请手动下载后重试"
      fi
    EOT
  }

  depends_on = [null_resource.create_namespace]
}

# 应用ArgoCD安装清单
resource "null_resource" "apply_argocd_manifest" {
  provisioner "local-exec" {
    command = <<-EOT
      kubectl apply -n ${var.namespace} -f /tmp/argocd/install.yaml
    EOT
  }

  provisioner "local-exec" {
    when    = destroy
    command = <<-EOT
      kubectl delete -n argocd -f /tmp/argocd/install.yaml || true
    EOT
  }

  depends_on = [
    null_resource.create_namespace,
    null_resource.prepare_argocd_manifest
  ]
}

# 等待ArgoCD服务器就绪
resource "null_resource" "wait_for_argocd" {
  count = var.wait_for_ready ? 1 : 0

  provisioner "local-exec" {
    command = <<-EOT
      echo "等待ArgoCD服务器就绪..."
      kubectl wait --for=condition=Ready pod -l app.kubernetes.io/name=argocd-server -n ${var.namespace} --timeout=600s
      echo "ArgoCD服务器已就绪"
    EOT
  }

  depends_on = [null_resource.apply_argocd_manifest]
}

# 配置ArgoCD服务类型 (如果需要LoadBalancer)
resource "null_resource" "configure_service_type" {
  count = var.service_type == "LoadBalancer" ? 1 : 0

  provisioner "local-exec" {
    command = <<-EOT
      kubectl patch svc argocd-server -n ${var.namespace} -p '{"spec":{"type":"LoadBalancer"}}'
    EOT
  }

  depends_on = [null_resource.apply_argocd_manifest]
}

# 配置ArgoCD为insecure模式 (如果启用)
resource "null_resource" "configure_insecure" {
  count = var.enable_insecure ? 1 : 0

  provisioner "local-exec" {
    command = <<-EOT
      kubectl patch deployment argocd-server -n ${var.namespace} --type='merge' -p='{"spec":{"template":{"spec":{"containers":[{"name":"argocd-server","args":["argocd-server","--insecure"]}]}}}}'
    EOT
  }

  depends_on = [null_resource.apply_argocd_manifest]
}

# 设置管理员密码
resource "null_resource" "set_admin_password" {
  provisioner "local-exec" {
    command = <<-EOT
      # 等待ArgoCD服务器完全启动
      kubectl wait --for=condition=Ready pod -l app.kubernetes.io/name=argocd-server -n ${var.namespace} --timeout=300s
      
      echo "正在设置ArgoCD管理员密码..."
      
      # 先删除初始admin secret (强制使用我们设置的密码)
      kubectl delete secret argocd-initial-admin-secret -n ${var.namespace} --ignore-not-found=true
      
      # 使用更简单的方法: 直接创建一个新的密码哈希
      # 使用ArgoCD Server Pod内的argocd CLI来生成哈希
      POD_NAME=$(kubectl get pods -n ${var.namespace} -l app.kubernetes.io/name=argocd-server -o jsonpath='{.items[0].metadata.name}')
      
      echo "使用ArgoCD CLI生成密码哈希..."
      BCRYPT_HASH=$(kubectl exec -n ${var.namespace} $POD_NAME -- argocd account bcrypt --password "${var.admin_password}" 2>/dev/null || echo "")
      
      # 如果ArgoCD CLI失败，使用htpasswd作为备选
      if [ -z "$BCRYPT_HASH" ]; then
        echo "ArgoCD CLI失败，使用htpasswd作为备选..."
        BCRYPT_HASH=$(htpasswd -bnBC 10 "" "${var.admin_password}" | cut -d: -f2)
      fi
      
      if [ -z "$BCRYPT_HASH" ]; then
        echo "错误: 无法生成bcrypt哈希"
        exit 1
      fi
      
      echo "更新ArgoCD secret..."
      # 更新argocd-secret中的admin.password
      kubectl patch secret argocd-secret -n ${var.namespace} --type='merge' -p="{\"data\":{\"admin.password\":\"$(echo -n "$BCRYPT_HASH" | base64 | tr -d '\n')\"}}"
      
      # 重启ArgoCD服务器使配置生效
      echo "重启ArgoCD服务器..."
      kubectl rollout restart deployment argocd-server -n ${var.namespace}
      kubectl rollout status deployment argocd-server -n ${var.namespace} --timeout=300s
      
      echo "======================================"
      echo "✅ ArgoCD管理员密码设置完成!"
      echo "用户名: admin"
      echo "密码: ${var.admin_password}"
      echo "======================================"
    EOT
  }

  depends_on = [
    null_resource.apply_argocd_manifest,
    null_resource.wait_for_argocd
  ]
}

# 随机ID用于资源唯一性
resource "random_id" "deployment_id" {
  byte_length = 4
}