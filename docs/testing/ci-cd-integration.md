# CI/CD 集成指南

**版本**: v1.0.0  
**最后更新**: 2024-12-20

---

## 目录

- [1. GitHub Actions配置](#1-github-actions配置)
- [2. 测试环境搭建](#2-测试环境搭建)
- [3. 测试执行流程](#3-测试执行流程)
- [4. 故障排查](#4-故障排查)
- [5. 最佳实践](#5-最佳实践)

---

## 1. GitHub Actions配置

### 1.1 工作流文件

**文件位置**: `.github/workflows/test.yml`

已配置的测试Job：
- ✅ **单元测试** (unit-tests) - 并行5个模块
- ✅ **安全测试** (security-tests) - 高风险访问点
- ✅ **集成测试** (integration-tests) - 完整服务栈
- ✅ **性能测试** (performance-tests) - k6负载测试（仅main分支）
- ✅ **代码质量** (code-quality) - SonarQube + Trivy

### 1.2 触发条件

```yaml
on:
  push:
    branches: [dev, main]
  pull_request:
    branches: [dev, main]
```

**自动触发场景**：
- 推送到 dev 或 main 分支
- 创建 Pull Request
- 更新 Pull Request

### 1.3 环境变量配置

**GitHub Secrets（需在仓库设置中配置）**：

| Secret名称 | 用途 | 示例值 |
|-----------|------|--------|
| `TEST_API_TOKEN` | 性能测试API Token | `Bearer eyJ...` |
| `SONAR_TOKEN` | SonarQube访问令牌 | `squ_xxxxx` |
| `SONAR_HOST_URL` | SonarQube服务器地址 | `https://sonarqube.example.com` |

**配置步骤**：

1. 进入 GitHub 仓库
2. Settings → Secrets and variables → Actions
3. 点击 "New repository secret"
4. 添加以上 Secrets

---

## 2. 测试环境搭建

### 2.1 本地测试环境

**启动完整测试环境**：

```bash
# 启动所有测试服务
docker-compose -f docker-compose.test.yml up -d

# 查看服务状态
docker-compose -f docker-compose.test.yml ps

# 查看日志
docker-compose -f docker-compose.test.yml logs -f

# 停止服务
docker-compose -f docker-compose.test.yml down
```

**服务清单**：
- PostgreSQL 15 (端口 5432)
- ClickHouse 23.8 (端口 8123, 9000)
- Redis 7 (端口 6379)
- Kafka 7.5.0 (端口 9092)
- Zookeeper (端口 2181)

### 2.2 数据库初始化

**PostgreSQL初始化**：

测试数据自动从 `tests/fixtures/init.sql` 加载。

**手动初始化**：

```bash
docker-compose -f docker-compose.test.yml exec postgres psql -U testuser -d hermesflow_test -f /docker-entrypoint-initdb.d/init.sql
```

**ClickHouse初始化**：

```bash
docker-compose -f docker-compose.test.yml exec clickhouse clickhouse-client --query "$(cat tests/fixtures/clickhouse_init.sql)"
```

### 2.3 健康检查

**验证服务就绪**：

```bash
# PostgreSQL
docker-compose -f docker-compose.test.yml exec postgres pg_isready -U testuser

# Redis
docker-compose -f docker-compose.test.yml exec redis redis-cli ping

# ClickHouse
curl http://localhost:8123/ping

# Kafka
docker-compose -f docker-compose.test.yml exec kafka kafka-broker-api-versions --bootstrap-server localhost:9092
```

---

## 3. 测试执行流程

### 3.1 本地执行测试

**运行所有测试**：

```bash
# 单元测试
pytest tests/unit -v

# 安全测试
pytest tests/security -v

# 集成测试
pytest tests/integration -v

# 性能测试
k6 run tests/performance/load_test.js
```

**运行特定测试文件**：

```bash
pytest tests/security/test_tenant_isolation.py -v
```

**运行特定测试用例**：

```bash
pytest tests/security/test_tenant_isolation.py::TestPostgreSQLRLS::test_rls_isolation_basic -v
```

**生成覆盖率报告**：

```bash
# Python
pytest --cov=. --cov-report=html

# Rust
cargo tarpaulin --out Html

# Java
mvn test jacoco:report
```

### 3.2 CI/CD执行流程

**完整流程图**：

```
1. 代码提交/PR创建
   ↓
2. GitHub Actions触发
   ↓
3. 并行执行单元测试（5个模块）
   ├── data-engine (Rust)
   ├── strategy-engine (Python)
   ├── trading-engine (Java)
   ├── user-management (Java)
   └── risk-engine (Java)
   ↓
4. 单元测试通过后，并行执行：
   ├── 安全测试
   └── 集成测试
   ↓
5. (仅main分支) 性能测试
   ↓
6. 代码质量检查（并行）
   ├── SonarQube扫描
   └── Trivy安全扫描
   ↓
7. 生成测试报告
   ↓
8. 通过/失败通知
```

**执行时间估算**：

| 阶段 | 预计时间 |
|------|---------|
| 单元测试 | 5-10分钟 |
| 安全测试 | 3-5分钟 |
| 集成测试 | 5-8分钟 |
| 性能测试 | 15-20分钟 |
| 代码质量 | 5-10分钟 |
| **总计** | **20-35分钟** |

### 3.3 测试报告

**GitHub Actions Summary**：

每次运行后自动生成测试摘要，包含：
- 测试通过/失败数量
- 覆盖率百分比
- 性能测试结果（如适用）

**Codecov集成**：

覆盖率自动上传到 Codecov：
- URL: `https://codecov.io/gh/{org}/{repo}`
- 按模块展示覆盖率
- PR中自动评论覆盖率变化

**JUnit XML报告**：

所有测试生成JUnit格式报告：
```
test-results/
├── unit-tests.xml
├── security-tests.xml
├── integration-tests.xml
└── performance-tests.xml
```

---

## 4. 故障排查

### 4.1 常见问题

#### 问题1：Docker服务启动失败

**症状**：
```
ERROR: for postgres  Cannot start service postgres: driver failed programming external connectivity
```

**原因**：端口被占用

**解决方案**：
```bash
# 查看端口占用
lsof -i :5432

# 停止占用端口的服务
sudo systemctl stop postgresql

# 或修改docker-compose.test.yml中的端口映射
ports:
  - "15432:5432"  # 使用15432替代5432
```

#### 问题2：测试数据库连接失败

**症状**：
```
psycopg2.OperationalError: could not connect to server: Connection refused
```

**原因**：数据库未就绪

**解决方案**：
```bash
# 等待数据库就绪
docker-compose -f docker-compose.test.yml up -d
sleep 30  # 等待30秒

# 或使用健康检查
until docker-compose -f docker-compose.test.yml exec postgres pg_isready -U testuser; do
  echo "Waiting for PostgreSQL..."
  sleep 2
done
```

#### 问题3：Kafka连接超时

**症状**：
```
KafkaTimeoutError: Failed to update metadata after 60.0 secs
```

**原因**：Kafka未完全启动

**解决方案**：
```bash
# 查看Kafka日志
docker-compose -f docker-compose.test.yml logs kafka

# 等待Kafka完全启动（需要Zookeeper先启动）
docker-compose -f docker-compose.test.yml up -d zookeeper
sleep 10
docker-compose -f docker-compose.test.yml up -d kafka
sleep 20
```

#### 问题4：测试覆盖率不达标

**症状**：
```
ERROR: Coverage 72% is below threshold 75%
```

**解决方案**：
```bash
# 查看未覆盖的代码
pytest --cov=. --cov-report=term-missing

# 或生成HTML报告
pytest --cov=. --cov-report=html
open htmlcov/index.html
```

### 4.2 调试技巧

**查看详细日志**：

```bash
# pytest详细输出
pytest -vv -s

# 显示print输出
pytest -s

# 只运行失败的测试
pytest --lf

# 失败时进入调试器
pytest --pdb
```

**查看Docker容器日志**：

```bash
# 所有服务日志
docker-compose -f docker-compose.test.yml logs

# 特定服务日志
docker-compose -f docker-compose.test.yml logs postgres

# 实时跟踪日志
docker-compose -f docker-compose.test.yml logs -f
```

**进入容器调试**：

```bash
# 进入PostgreSQL容器
docker-compose -f docker-compose.test.yml exec postgres bash

# 连接数据库
docker-compose -f docker-compose.test.yml exec postgres psql -U testuser -d hermesflow_test

# 进入Redis容器
docker-compose -f docker-compose.test.yml exec redis redis-cli
```

---

## 5. 最佳实践

### 5.1 编写测试

**DO**：
- ✅ 每个测试独立（无依赖）
- ✅ 使用描述性的测试名称
- ✅ 测试一个功能点
- ✅ 使用Fixtures管理测试数据
- ✅ 清理测试数据（使用事务回滚）

**DON'T**：
- ❌ 测试间共享状态
- ❌ 依赖测试执行顺序
- ❌ 硬编码测试数据
- ❌ 忽略测试失败
- ❌ 跳过测试（除非有充分理由）

### 5.2 CI/CD优化

**加速测试执行**：

```yaml
# 使用缓存
- name: Cache Python dependencies
  uses: actions/cache@v3
  with:
    path: ~/.cache/pip
    key: ${{ runner.os }}-pip-${{ hashFiles('**/requirements.txt') }}

# 并行执行
strategy:
  matrix:
    module: [data-engine, strategy-engine, ...]
  fail-fast: false  # 不因单个失败停止所有测试
```

**减少资源消耗**：

```yaml
# 只在必要时运行性能测试
if: github.ref == 'refs/heads/main'

# 限制并发
concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true
```

### 5.3 测试维护

**定期审查**：
- 每月审查测试覆盖率报告
- 识别未覆盖的代码路径
- 删除过时的测试
- 更新测试数据

**持续改进**：
- 监控测试执行时间
- 优化慢测试
- 添加新功能的测试
- 重构脆弱的测试

---

## 附录

### A. 有用的命令

```bash
# 清理测试环境
docker-compose -f docker-compose.test.yml down -v
rm -rf test-results/ htmlcov/ .pytest_cache/

# 重新构建测试镜像
docker-compose -f docker-compose.test.yml build --no-cache

# 查看资源使用
docker stats

# 导出测试数据
docker-compose -f docker-compose.test.yml exec postgres pg_dump -U testuser hermesflow_test > backup.sql
```

### B. 参考链接

- [GitHub Actions文档](https://docs.github.com/en/actions)
- [pytest文档](https://docs.pytest.org/)
- [Docker Compose文档](https://docs.docker.com/compose/)
- [k6性能测试文档](https://k6.io/docs/)
- [Codecov文档](https://docs.codecov.com/)

---

**最后更新**: 2024-12-20  
**维护团队**: QA Team

