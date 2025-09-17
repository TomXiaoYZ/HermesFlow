# HermesFlow 数据库DDL管理

- 所有DDL文件统一存放于本目录，按数据库类型和环境分类。
- 通用DDL放于 db/common/，各环境专用DDL放于 db/local/、db/dev/、db/prod/。
- 命名规范：Vxxx__desc.sql，如 V001__init_user.sql。
- 变更需同步 docs/db-changelog.md。 