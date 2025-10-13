# 归档文档说明

> **目的**: 保存历史文档，保持主文档区域整洁

---

## 📋 归档策略

### 什么文档需要归档？

1. **过时的版本文档**
   - 旧版本的报告（如 Master Checklist V1, V2）
   - 已被新版本替代的文档
   
2. **已完成的临时文档**
   - 一次性的分析报告
   - 已实施的迁移计划
   
3. **实验性文档**
   - 未被采纳的技术方案
   - POC（概念验证）文档

### 什么文档不应归档？

- **当前版本的文档** - 保留在主文档区
- **历史参考价值高的文档** - 保留在主文档区
- **频繁引用的文档** - 保留在主文档区

---

## 📂 归档结构

```
docs/archived/
├── README.md (本文件)
├── reports/              # 报告归档
│   ├── master-checklist-report-v1.md
│   └── master-checklist-report-v2.md
├── prd/                  # PRD归档
│   └── (旧版本PRD)
├── architecture/         # 架构归档
│   └── (已废弃的架构方案)
└── migrations/           # 迁移文档归档
    └── (已完成的迁移计划)
```

---

## 🔄 归档流程

### 1. 确认归档

在归档前，确认：
- [ ] 文档确实已过时或不再需要
- [ ] 有更新的版本替代
- [ ] 团队同意归档

### 2. 执行归档

```bash
# 使用 git mv 保留历史
git mv docs/OLD-DOC.md docs/archived/category/old-doc.md

# 示例：归档旧报告
git mv docs/MASTER-CHECKLIST-REPORT.md docs/archived/reports/master-checklist-report-v1.md
```

### 3. 更新引用

- 更新主文档中的链接
- 在归档文档开头添加归档说明
- 在新文档中添加"替代文档"说明

### 4. 文档说明

在归档文档顶部添加归档说明：

```markdown
> **⚠️ 已归档**: 本文档已于 YYYY-MM-DD 归档  
> **原因**: [归档原因]  
> **替代文档**: [新文档链接]
```

---

## 📜 归档历史

### 2025-01-13 - 文档标准化清理

**归档内容**:
- `master-checklist-report-v1.md` - 主检查清单报告 V1.0
- `master-checklist-report-v2.md` - 主检查清单报告 V2.0

**原因**: 创建V3.0报告，旧版本归档保留历史

**替代文档**: 
- `docs/master-checklist-report-v3.md` (待创建)

---

## 🔍 查找归档文档

### 按类别浏览

```bash
# 报告
ls docs/archived/reports/

# PRD
ls docs/archived/prd/

# 架构
ls docs/archived/architecture/
```

### 全局搜索

```bash
# 在归档区搜索
grep -r "关键词" docs/archived/

# 示例
grep -r "Master Checklist" docs/archived/
```

---

## 🗑️ 删除策略

### 保留期限

- **报告**: 至少保留1年
- **PRD**: 至少保留2年
- **架构文档**: 至少保留2年
- **迁移文档**: 迁移完成后保留1年

### 删除流程

过期后，经团队同意可以删除：

```bash
# 1. 确认文档已过保留期
# 2. 在团队会议中提出删除请求
# 3. 获得批准后删除

git rm docs/archived/old-file.md
git commit -m "chore: remove expired archived document"
```

---

## 📊 归档统计

| 类别 | 文档数量 | 最早归档日期 | 最新归档日期 |
|------|---------|-------------|-------------|
| 报告 | 2 | 2025-01-13 | 2025-01-13 |
| PRD | 0 | - | - |
| 架构 | 0 | - | - |
| 迁移 | 0 | - | - |

---

## 📞 联系

如有关于归档文档的问题：
- **负责人**: @po.mdc
- **Slack**: #hermesflow-docs
- **Email**: docs@hermesflow.example

---

**最后更新**: 2025-01-13  
**维护者**: @po.mdc

