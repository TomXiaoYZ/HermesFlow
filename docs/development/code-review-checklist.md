# 代码审查清单

> **确保代码质量的完整检查清单** | **适用于**: Rust, Java, Python

---

## 🎯 使用方法

### 作为代码提交者（Author）
在提交 Pull Request **之前**，使用本清单进行自查。

### 作为代码审查者（Reviewer）
在审查 Pull Request 时，使用本清单确保全面审查。

---

## ✅ 通用检查项（所有语言）

### 1. 代码质量

#### 可读性
- [ ] **命名清晰**: 变量、函数、类名能够自解释
- [ ] **函数职责单一**: 每个函数只做一件事
- [ ] **代码长度合理**: 
  - 函数/方法 < 50 行
  - 类 < 500 行
- [ ] **注释恰当**: 
  - 复杂逻辑有注释说明
  - 避免过多注释（代码应自解释）
- [ ] **格式一致**: 遵循项目编码规范

#### 复杂度
- [ ] **避免深层嵌套**: 嵌套层级 ≤ 3
- [ ] **循环复杂度低**: McCabe 复杂度 < 10
- [ ] **避免魔法数字**: 使用常量代替硬编码值
- [ ] **DRY 原则**: 消除重复代码

#### 错误处理
- [ ] **异常处理完整**: 
  - 所有可能的错误都被处理
  - 不吞噬异常
  - 提供有意义的错误消息
- [ ] **资源管理**: 
  - 文件、连接、锁等资源正确释放
  - 使用 RAII / try-with-resources / context manager

### 2. 功能正确性

- [ ] **需求符合**: 代码实现符合 PRD 和 Story 要求
- [ ] **验收标准满足**: 所有验收标准都已实现
- [ ] **边界条件处理**: 
  - 空值/null/None 处理
  - 边界值测试
  - 极端情况处理
- [ ] **并发安全**: 
  - 多线程场景下的数据竞争
  - 锁的正确使用
  - 死锁风险评估

### 3. 性能

- [ ] **无明显性能问题**: 
  - 避免 N+1 查询
  - 合理的算法复杂度（时间/空间）
  - 避免不必要的内存分配
- [ ] **资源使用合理**: 
  - 内存占用
  - CPU 使用
  - 网络带宽
- [ ] **缓存使用得当**: 
  - 合理使用 Redis 缓存
  - 避免缓存穿透/雪崩

### 4. 安全性

- [ ] **SQL 注入防护**: 
  - 使用参数化查询
  - 避免字符串拼接 SQL
- [ ] **XSS 防护**: 
  - 用户输入转义
  - 输出编码
- [ ] **认证和授权**: 
  - JWT Token 验证
  - RBAC 权限检查
- [ ] **敏感信息保护**: 
  - 不在日志中打印密码/Token
  - 敏感配置使用环境变量或密钥管理
- [ ] **多租户隔离**: 
  - PostgreSQL RLS 正确使用
  - Redis Key 带租户前缀
  - 查询都包含 tenant_id 过滤

### 5. 测试

- [ ] **单元测试完整**: 
  - 覆盖率达标（Rust≥85%, Java≥80%, Python≥75%）
  - 测试关键路径和边界条件
- [ ] **测试可读**: 
  - 测试名称清晰
  - 使用 Arrange-Act-Assert 模式
- [ ] **Mock 使用合理**: 
  - 外部依赖 Mock
  - 避免过度 Mock
- [ ] **集成测试**（如适用）: 
  - 跨模块集成测试
  - 数据库集成测试

### 6. 文档

- [ ] **API 文档更新**: 
  - OpenAPI/gRPC 规范更新
  - 示例代码更新
- [ ] **README 更新**: 
  - 新功能使用说明
  - 配置项说明
- [ ] **注释更新**: 
  - 删除过时注释
  - 更新修改的逻辑注释

### 7. Git 和提交

- [ ] **Commit 消息规范**: 
  - 使用 Conventional Commits
  - 示例: `feat(data-engine): add Binance connector`
- [ ] **Commit 粒度合理**: 
  - 每个 commit 是一个独立的逻辑单元
  - 避免"WIP"或"fix typo"的 commit
- [ ] **分支策略**: 
  - 从正确的分支创建（develop）
  - 分支命名规范（feat/xxx, fix/xxx）

### 8. CI/CD

- [ ] **CI Pipeline 通过**: 
  - 所有测试通过
  - Linter 通过
  - Security Scan 通过
- [ ] **构建成功**: 
  - Docker 镜像构建成功
  - 无构建警告

---

## 🦀 Rust 特定检查项

### 代码风格

- [ ] **使用 `cargo fmt` 格式化**: 
  ```bash
  cargo fmt --check
  ```
- [ ] **通过 `cargo clippy`**: 
  ```bash
  cargo clippy -- -D warnings
  ```
- [ ] **遵循 Rust 命名约定**: 
  - snake_case for functions, variables, modules
  - CamelCase for types, traits
  - SCREAMING_SNAKE_CASE for constants

### 所有权和生命周期

- [ ] **所有权清晰**: 
  - 避免不必要的 `clone()`
  - 合理使用引用（&）和可变引用（&mut）
- [ ] **生命周期正确**: 
  - 生命周期标注清晰
  - 避免悬垂引用
- [ ] **避免 `unsafe`**: 
  - 除非绝对必要（如 FFI）
  - 如使用 `unsafe`，必须有注释说明安全性

### 错误处理

- [ ] **使用 `Result<T, E>` 而非 `panic!`**: 
  - 库代码不应 panic
  - 使用 `?` 操作符传播错误
- [ ] **自定义错误类型**: 
  - 使用 `thiserror` 或 `anyhow`
  - 提供有意义的错误消息

### 异步代码

- [ ] **正确使用 `async/await`**: 
  - 避免阻塞异步运行时（不在 async 中调用同步阻塞函数）
  - 使用 `tokio::spawn_blocking` 处理 CPU 密集任务
- [ ] **并发安全**: 
  - 使用 `Arc<Mutex<T>>` 或 `Arc<RwLock<T>>`
  - 避免死锁

### 性能

- [ ] **避免不必要的分配**: 
  - 使用 `&str` 而非 `String`（如不需要所有权）
  - 使用 `Vec::with_capacity` 预分配
- [ ] **迭代器链式调用**: 
  - 使用 `.iter()` 而非索引
  - 使用 `.collect()` 代替手动 push

### 依赖

- [ ] **`Cargo.toml` 版本合理**: 
  - 使用 `^` 版本（如 `tokio = "^1.35"`）
  - 审查新增依赖的必要性

---

## ☕ Java 特定检查项

### 代码风格

- [ ] **遵循 Google Java Style Guide**: 
  ```bash
  ./mvnw checkstyle:check
  ```
- [ ] **使用 IDE 格式化**: 
  - IntelliJ: Ctrl+Alt+L
  - Eclipse: Ctrl+Shift+F

### Spring Boot 最佳实践

- [ ] **使用构造函数注入**: 
  ```java
  // Good
  private final UserService userService;
  
  public UserController(UserService userService) {
      this.userService = userService;
  }
  
  // Bad: @Autowired field injection
  ```
- [ ] **使用 `@Validated` 验证**: 
  ```java
  @PostMapping("/users")
  public ResponseEntity<User> createUser(@Valid @RequestBody UserRequest request) {
      ...
  }
  ```
- [ ] **异常处理统一**: 
  - 使用 `@ControllerAdvice` 全局异常处理
  - 返回统一的错误响应格式

### JDK 21 Virtual Threads

- [ ] **合理使用 Virtual Threads**: 
  ```java
  // application.yml
  spring:
    threads:
      virtual:
        enabled: true
  ```
- [ ] **避免 Pinning**: 
  - 不在 synchronized 块中长时间阻塞
  - 使用 ReentrantLock 代替 synchronized

### JPA 最佳实践

- [ ] **避免 N+1 查询**: 
  ```java
  // Good: 使用 JOIN FETCH
  @Query("SELECT u FROM User u JOIN FETCH u.orders WHERE u.id = :id")
  User findByIdWithOrders(@Param("id") Long id);
  ```
- [ ] **使用分页**: 
  ```java
  Page<User> findAll(Pageable pageable);
  ```
- [ ] **事务边界清晰**: 
  - Service 层使用 `@Transactional`
  - 只读操作使用 `@Transactional(readOnly = true)`

### 多租户

- [ ] **RLS 上下文正确设置**: 
  ```java
  @PrePersist
  public void setTenantId() {
      this.tenantId = TenantContext.getCurrentTenantId();
  }
  ```
- [ ] **所有查询都包含 tenant_id**: 
  ```java
  @Query("SELECT o FROM Order o WHERE o.tenantId = :tenantId")
  List<Order> findByTenantId(@Param("tenantId") String tenantId);
  ```

---

## 🐍 Python 特定检查项

### 代码风格

- [ ] **遵循 PEP 8**: 
  ```bash
  poetry run pylint src/
  ```
- [ ] **使用 `black` 格式化**: 
  ```bash
  poetry run black --check src/
  ```
- [ ] **Type Hints**: 
  ```python
  def calculate_rsi(prices: List[float], period: int = 14) -> float:
      ...
  ```

### FastAPI 最佳实践

- [ ] **使用 Pydantic 模型**: 
  ```python
  from pydantic import BaseModel
  
  class UserRequest(BaseModel):
      username: str
      email: EmailStr
  ```
- [ ] **依赖注入**: 
  ```python
  @app.get("/users/{user_id}")
  async def get_user(
      user_id: int,
      db: Session = Depends(get_db)
  ):
      ...
  ```
- [ ] **异步处理**: 
  ```python
  # 使用 async def 处理 I/O 密集任务
  async def fetch_market_data(symbol: str) -> MarketData:
      async with httpx.AsyncClient() as client:
          ...
  ```

### NumPy/Pandas 优化

- [ ] **向量化计算**: 
  ```python
  # Good: 向量化
  rsi = (up_avg / (up_avg + down_avg)) * 100
  
  # Bad: 循环
  for i in range(len(prices)):
      rsi[i] = calculate_single_rsi(prices[i])
  ```
- [ ] **避免 DataFrame 逐行迭代**: 
  ```python
  # Good: apply()
  df['rsi'] = df.apply(lambda row: calculate_rsi(row['price']), axis=1)
  
  # Bad: iterrows()
  for index, row in df.iterrows():  # 慢！
      ...
  ```

### 错误处理

- [ ] **使用自定义异常**: 
  ```python
  class StrategyError(Exception):
      """策略执行错误"""
      pass
  ```
- [ ] **记录日志**: 
  ```python
  import logging
  logger = logging.getLogger(__name__)
  
  logger.error(f"Failed to execute strategy: {str(e)}")
  ```

### 测试

- [ ] **使用 pytest fixtures**: 
  ```python
  @pytest.fixture
  def market_data():
      return MarketDataFactory.create()
  ```
- [ ] **异步测试**: 
  ```python
  @pytest.mark.asyncio
  async def test_fetch_data():
      data = await fetch_market_data("BTC/USDT")
      assert data is not None
  ```

---

## 📋 Code Review 流程

### 1. 自查（Author）

在创建 PR 之前：

```bash
# 1. 运行 Linter
cargo clippy -- -D warnings  # Rust
./mvnw checkstyle:check      # Java
poetry run pylint src/       # Python

# 2. 运行测试
cargo test                   # Rust
./mvnw test                  # Java
poetry run pytest            # Python

# 3. 检查覆盖率
cargo tarpaulin             # Rust
./mvnw jacoco:report        # Java
poetry run pytest --cov     # Python

# 4. 运行格式化
cargo fmt                   # Rust
# IntelliJ 自动格式化      # Java
poetry run black src/       # Python

# 5. 本地运行服务
cargo run                   # Rust
./mvnw spring-boot:run      # Java
poetry run python main.py   # Python
```

使用本清单逐项检查，确保所有项都通过。

### 2. 创建 PR

PR 描述模板：

```markdown
## 📝 描述
[简要描述这个 PR 做了什么]

## 🔗 相关 Issue
Closes #xxx

## 🎯 变更类型
- [ ] feat: 新功能
- [ ] fix: Bug 修复
- [ ] docs: 文档更新
- [ ] test: 测试相关
- [ ] refactor: 代码重构
- [ ] perf: 性能优化

## ✅ 自查清单
- [ ] 代码符合编码规范
- [ ] 通过了所有测试
- [ ] 测试覆盖率达标（Rust≥85%, Java≥80%, Python≥75%）
- [ ] 更新了相关文档
- [ ] 通过了 CI/CD 检查
- [ ] 无 Security Scan 漏洞

## 📸 截图（如适用）
[添加截图或 GIF]

## 🧪 测试计划
[如何测试这个 PR]
```

### 3. 审查（Reviewer）

审查步骤：

1. **快速浏览**（5分钟）
   - [ ] PR 描述清晰
   - [ ] 变更规模合理（< 500 行）
   - [ ] CI/CD 通过

2. **深入审查**（15-30分钟）
   - [ ] 逐文件审查代码
   - [ ] 使用本清单检查
   - [ ] 提出具体的改进建议

3. **功能验证**（10分钟）
   - [ ] Checkout 到 PR 分支
   - [ ] 本地运行和测试
   - [ ] 验证功能正确性

4. **提供反馈**
   - 使用建设性语言
   - 区分"必须修改"和"建议"
   - 认可好的实践

### 4. 响应反馈（Author）

```bash
# 1. 修改代码
vim src/your_file.rs

# 2. 提交修改
git add .
git commit -m "refactor: address code review feedback"
git push origin your-branch

# 3. 回复每条评论
# 在 GitHub 上回复，说明修改或解释
```

### 5. 批准和合并（Reviewer）

- [ ] 所有反馈已解决
- [ ] CI/CD 通过
- [ ] 至少 1 人批准（Approve）
- [ ] 点击 "Merge" 按钮

---

## ⏱️ 时间目标

| 活动 | 目标时间 |
|------|---------|
| 自查 | < 15 分钟 |
| Code Review | < 4 小时（从 PR 创建到第一次反馈） |
| 响应反馈 | < 2 小时 |
| 批准和合并 | < 1 小时 |
| **总计** | **< 1 工作日** |

---

## 📞 获取帮助

### Code Review 问题
- Slack: `#code-review`
- 文档: [开发指南](./dev-guide.md)

### 编码规范疑问
- 文档: [编码规范](./coding-standards.md)

### 测试问题
- 文档: [测试策略](../testing/test-strategy.md)
- Slack: `#qa-team`

---

**最后更新**: 2025-01-13  
**维护者**: @pm.mdc  
**版本**: v1.0

