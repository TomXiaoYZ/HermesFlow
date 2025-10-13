# 文档命名规范

> **版本**: v1.0 | **更新日期**: 2025-01-13

本文档定义了 HermesFlow 项目文档的命名标准，确保文档命名的一致性和可维护性。

---

## 📋 命名规则

### 1. 基本规则

- **全部小写**: 所有文档文件名使用小写字母
- **连字符分隔**: 使用连字符（`-`）分隔单词（kebab-case）
- **有意义的名称**: 文件名应清晰描述内容

### 2. 例外情况

- **README.md**: 约定俗成，保持大写
- **LICENSE**: 如有许可证文件，保持大写

---

## ✅ 正确示例

### 文档文件名

```
✅ system-architecture.md
✅ api-design.md
✅ qa-engineer-guide.md
✅ sprint-planning-checklist.md
✅ code-review-checklist.md
✅ developer-quickstart.md
```

### 目录名

```
✅ docs/
✅ prd/
✅ architecture/
✅ development/
✅ testing/
✅ operations/
```

---

## ❌ 错误示例

```
❌ SYSTEM-ARCHITECTURE.md      # 全大写
❌ API-Design.md                # 混合大小写
❌ QA-ENGINEER-GUIDE.md         # 大写缩写
❌ Sprint_Planning_Checklist.md # 使用下划线
❌ CodeReviewChecklist.md       # 驼峰命名
❌ code_review_checklist.md     # 使用下划线（应用连字符）
```

---

## 📂 特殊文件类型

### 缩写处理

**规则**: 缩写全部小写

```
✅ api-design.md       # 不是 API-design.md
✅ qa-engineer-guide.md # 不是 QA-engineer-guide.md
✅ devops-guide.md     # 不是 DevOps-guide.md
✅ cicd-flow.md        # 不是 CICD-flow.md
✅ prd-hermesflow.md   # 不是 PRD-HermesFlow.md
```

### 数字编号

**规则**: 使用前导零保持对齐

```
✅ 01-data-module.md
✅ 02-strategy-module.md
...
✅ 10-frontend-module.md
```

### ADR (Architecture Decision Record)

**规则**: `adr-编号-描述.md`

```
✅ adr-001-hybrid-tech-stack.md
✅ adr-002-multi-tenancy-architecture.md
✅ adr-003-message-communication.md
```

---

## 🔄 重命名流程

### 使用 Git 重命名（保留历史）

```bash
# 单个文件
git mv OLD-NAME.md new-name.md

# 示例
git mv QUICKSTART.md quickstart.md
git mv FAQ.md faq.md
```

### 批量重命名

```bash
# 使用脚本批量重命名
for file in *.md; do
  newname=$(echo "$file" | tr 'A-Z' 'a-z' | tr '_' '-')
  [ "$file" != "$newname" ] && git mv "$file" "$newname"
done
```

---

## 📝 更新链接

重命名文件后，必须更新所有引用该文件的链接：

### 查找引用

```bash
# 查找所有引用
grep -r "OLD-NAME.md" docs/

# 示例
grep -r "QUICKSTART.md" docs/
```

### 批量替换

```bash
# 使用 sed 批量替换
sed -i '' 's|OLD-NAME\.md|new-name.md|g' docs/**/*.md

# 示例
sed -i '' 's|QUICKSTART\.md|quickstart.md|g' docs/**/*.md
```

---

## ✅ 验证清单

重命名后，检查以下内容：

- [ ] 所有文件名符合命名规范
- [ ] 无混合大小写的文件名
- [ ] 无使用下划线的文件名（除非特殊需要）
- [ ] 所有内部链接已更新
- [ ] 所有外部引用已通知
- [ ] Git历史保留（使用`git mv`）
- [ ] 文档导航（README.md）已更新

---

## 📚 相关文档

- [文档导航](./README.md)
- [项目进度](./progress.md)
- [归档策略](./archived/README.md)

---

## 🔧 维护

### 审查频率

- **每月**: 运行命名规范检查
- **每 Sprint**: Code Review 时检查新文档

### 检查脚本

```bash
#!/bin/bash
# check-naming.sh - 检查文档命名规范

echo "检查不符合命名规范的文件..."

# 查找大写文件（排除 README.md 和 LICENSE）
find docs -type f -name "*.md" ! -name "README.md" ! -name "LICENSE" | while read file; do
  filename=$(basename "$file")
  if [[ "$filename" =~ [A-Z] ]]; then
    echo "❌ 不符合规范: $file"
  fi
  if [[ "$filename" =~ _ ]]; then
    echo "❌ 使用下划线: $file"
  fi
done

echo "检查完成"
```

---

**维护者**: @po.mdc, @architect.mdc  
**审查周期**: 每月  
**最后审查**: 2025-01-13

