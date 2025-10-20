#!/bin/bash

# 切换到项目目录
cd /Users/tomxiao/Desktop/Git/personal/HermesFlow

# 记录输出
LOGFILE="/tmp/hermesflow-cicd-commit-$(date +%s).log"
echo "=== Starting commit and push process ===" > "$LOGFILE"
echo "Timestamp: $(date)" >> "$LOGFILE"
echo "" >> "$LOGFILE"

# 检查当前分支
echo "=== Current Branch ===" >> "$LOGFILE"
git branch --show-current >> "$LOGFILE" 2>&1
echo "" >> "$LOGFILE"

# 添加所有更改
echo "=== Git Add ===" >> "$LOGFILE"
git add -A >> "$LOGFILE" 2>&1
echo "" >> "$LOGFILE"

# 检查状态
echo "=== Git Status ===" >> "$LOGFILE"
git status --short >> "$LOGFILE" 2>&1
echo "" >> "$LOGFILE"

# 提交
echo "=== Git Commit ===" >> "$LOGFILE"
git commit -m "[module: data-engine] 测试基于commit message的CI/CD流程

- 添加测试标记文件
- 验证parse-commit job是否正确解析模块名称
- 验证只构建data-engine模块
- 验证GitOps自动更新" >> "$LOGFILE" 2>&1
COMMIT_STATUS=$?
echo "Commit exit code: $COMMIT_STATUS" >> "$LOGFILE"
echo "" >> "$LOGFILE"

# 推送（带重试）
echo "=== Git Push with Retry ===" >> "$LOGFILE"
for i in {1..5}; do
    echo "Push attempt $i..." >> "$LOGFILE"
    if git push origin develop >> "$LOGFILE" 2>&1; then
        echo "✅ Push successful!" >> "$LOGFILE"
        echo "SUCCESS" >> "$LOGFILE"
        break
    else
        echo "⚠️ Push attempt $i failed" >> "$LOGFILE"
        if [ $i -lt 5 ]; then
            echo "Retrying in 10 seconds..." >> "$LOGFILE"
            sleep 10
        else
            echo "❌ All push attempts failed" >> "$LOGFILE"
            echo "FAILED" >> "$LOGFILE"
        fi
    fi
done

echo "" >> "$LOGFILE"
echo "=== Process Complete ===" >> "$LOGFILE"
echo "Log file: $LOGFILE" >> "$LOGFILE"

# 输出日志文件位置
echo "$LOGFILE"

