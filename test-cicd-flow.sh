#!/bin/bash

# CI/CD 流程测试脚本
# 用于测试基于 commit message 的模块选择性构建

set -e

echo "====================================="
echo "CI/CD 流程测试脚本"
echo "====================================="
echo ""

# 检查当前分支
CURRENT_BRANCH=$(git branch --show-current)
echo "当前分支: $CURRENT_BRANCH"

if [[ "$CURRENT_BRANCH" != "develop" ]] && [[ "$CURRENT_BRANCH" != "main" ]]; then
    echo "⚠️  警告: 当前不在 develop 或 main 分支上"
    echo "CI/CD 只在 develop 和 main 分支上触发"
    read -p "是否切换到 develop 分支? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        git checkout develop || git checkout -b develop
    else
        echo "❌ 取消测试"
        exit 1
    fi
fi

echo ""
echo "支持的模块列表:"
echo "  1. data-engine (Rust)"
echo "  2. gateway (Rust)"
echo "  3. user-management (Java)"
echo "  4. api-gateway (Java)"
echo "  5. trading-engine (Java)"
echo "  6. strategy-engine (Python)"
echo "  7. backtest-engine (Python)"
echo "  8. risk-engine (Python)"
echo "  9. frontend (React/TypeScript)"
echo "  10. all (构建所有模块)"
echo ""

read -p "请选择要测试的模块 (输入模块名称或数字): " MODULE_INPUT

# 映射数字到模块名称
case $MODULE_INPUT in
    1) MODULE="data-engine";;
    2) MODULE="gateway";;
    3) MODULE="user-management";;
    4) MODULE="api-gateway";;
    5) MODULE="trading-engine";;
    6) MODULE="strategy-engine";;
    7) MODULE="backtest-engine";;
    8) MODULE="risk-engine";;
    9) MODULE="frontend";;
    10) MODULE="all";;
    *) MODULE="$MODULE_INPUT";;
esac

echo ""
echo "📝 准备提交测试..."
echo "   模块: $MODULE"
echo "   分支: $(git branch --show-current)"
echo ""

# 创建一个测试文件
TEST_FILE=".cicd-test-$(date +%s).tmp"
echo "Test commit at $(date)" > "$TEST_FILE"
git add "$TEST_FILE"

# 提交
COMMIT_MSG="[module: $MODULE] 测试 CI/CD 流程 - $(date '+%Y-%m-%d %H:%M:%S')"
echo "提交信息: $COMMIT_MSG"
git commit -m "$COMMIT_MSG"

# 推送（带重试）
echo ""
echo "🚀 正在推送到远程仓库..."
for i in {1..5}; do
    if git push origin $(git branch --show-current); then
        echo "✅ 成功推送到远程仓库"
        break
    else
        echo "⚠️  推送失败，尝试 $i/5..."
        if [ $i -eq 5 ]; then
            echo "❌ 推送失败，请检查网络连接"
            exit 1
        fi
        sleep 10
    fi
done

echo ""
echo "✅ 测试提交已推送！"
echo ""
echo "📊 下一步操作："
echo "1. 访问 GitHub Actions 查看 CI 运行状态:"
echo "   https://github.com/TomXiaoYZ/HermesFlow/actions"
echo ""
echo "2. 预期结果:"
if [ "$MODULE" = "all" ]; then
    echo "   - 所有相关的 CI workflows 应该被触发"
    echo "   - 所有模块都应该被构建"
else
    echo "   - 对应的 CI workflow 应该被触发"
    echo "   - 只有 $MODULE 模块应该被构建"
fi
echo "   - Docker 镜像应该被推送到 ACR"
echo "   - HermesFlow-GitOps 仓库应该被自动更新"
echo "   - ArgoCD 应该自动同步并部署更新"
echo ""
echo "3. 验证 GitOps 更新:"
echo "   cd ../HermesFlow-GitOps"
echo "   git pull origin main"
echo "   # 检查 apps/dev/$MODULE/values.yaml 中的 image.tag"
echo ""
echo "4. 验证 ArgoCD 部署:"
echo "   kubectl get pods -n hermesflow-dev"
echo "   kubectl describe pod <pod-name> -n hermesflow-dev"
echo ""

# 清理测试文件
git rm "$TEST_FILE"
git commit -m "chore: 清理 CI/CD 测试文件"
git push origin $(git branch --show-current)

echo "✅ 测试完成！"

