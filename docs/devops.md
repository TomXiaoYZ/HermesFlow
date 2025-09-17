# HermesFlow DevOps与自动化运维手册

## 1. 脚本体系与自动化部署
### 1.1 目录结构
- 所有自动化脚本统一放置于 `scripts/` 目录。

### 1.2 主控脚本 deploy.sh
- 用法: `./deploy.sh --env [local|dev|prod] --side [front|back|all] --service [web|gateway|api-gateway|all]`
- 功能：
  - 前端：自动切换 .env，安装依赖、构建、（可选）上传、启动本地服务
  - 后端（Java）：自动切换 profile，编译、打包、（可选）上传、启动服务
  - 后端（Python）：自动切换 config，安装依赖、启动服务
  - 数据库：可选参数支持初始化/迁移 DDL
- 支持调用子脚本（如 deploy_front.sh、deploy_back_java.sh、db_init.sh）
- 日志输出、错误处理、参数校验

### 1.3 新增服务/变量流程
- 新增服务：
  1. 在`scripts/`目录补充相关脚本或在主控脚本中注册新服务。
  2. 在`db/`目录补充相关DDL。
  3. 在本文件相关章节补充说明。
- 新增变量：
  1. 在对应的.env/application/config文件补充变量。
  2. 在本文件环境变量章节登记变量说明。

---

## 2. 环境变量管理
### 2.1 前端环境变量
- 采用 .env.local/.env.dev/.env.prod 文件，变量如：
  - VITE_API_BASE_URL：API服务地址
  - VITE_ENV_NAME：环境名称

### 2.2 Java服务环境变量
- 采用 Spring Boot application-local.yml/application-dev.yml/application-prod.yml 多profile管理。
- 常见变量如：
  - spring.datasource.url：数据库连接
  - spring.redis.host：Redis主机
  - custom.api.base-url：后端API地址

### 2.3 Python服务环境变量
- 采用 config/env.local.yaml 等，变量如：
  - DB_HOST、DB_PORT、API_BASE_URL 等

### 2.4 新增/变更流程
- 新增变量：在对应配置文件和本文件同步补充说明。
- 变更变量：需Pull Request审核，确保文档与代码同步。

---

## 3. 数据库DDL管理
### 3.1 目录结构
- db/common/：通用DDL
- db/local/、db/dev/、db/prod/：各环境专用DDL
- db/[dbtype]/：按数据库类型分类（如postgres、clickhouse、redis等）

### 3.2 命名规范
- 每个DDL文件命名为 Vxxx__desc.sql，如 V001__init_user.sql

### 3.3 变更记录
- 每次DDL变更需在此登记：
  - 变更日期
  - 变更人
  - 变更内容简述
  - 涉及DDL文件
  - 适用环境

#### 示例
- 2024-06-XX by tomxiao
  - 新增用户表DDL
  - 文件：db/common/postgres/V001__init_user.sql
  - 适用环境：all

---

## 4. FAQ与常见问题
- 如何新增服务？见1.3
- 如何切换环境？见2.1/2.2/2.3
- 如何初始化数据库？见3.1/3.2
- 其他问题请补充... 

## 5. 本地开发环境部署（local环境）
- 本地开发环境下，所有基础设施和业务服务均通过docker-compose统一以容器方式运行。
- 启动命令：
  ```bash
  ./scripts/deploy.sh --env local
  # 或直接
  docker-compose -f docker-compose.local.yml up -d
  ```
- 不再支持本地直接启动服务，确保环境一致性。
- 所有服务、端口、依赖详见 docker-compose.local.yml 文件头部注释。
- 如需扩展服务或自定义配置，可编辑 docker-compose.local.yml。 

### 5.1 deploy.sh 支持的命令
- up：启动所有服务（docker compose）
- down：停止所有服务
- status：查看服务状态
- logs：查看所有服务日志（Ctrl+C退出）
- clean：停止并清理所有容器和数据卷
- help：显示帮助信息

#### 用法示例
```bash
./scripts/deploy.sh --env local up
./scripts/deploy.sh --env local down
./scripts/deploy.sh --env local status
./scripts/deploy.sh --env local logs
./scripts/deploy.sh --env local clean
./scripts/deploy.sh --env local help
```
- dev/prod 环境暂未实现自动部署，后续可扩展上传、远程部署等逻辑。 