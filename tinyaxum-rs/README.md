# hdata-analysis-rs

hdata-analysis 平台的后端服务:基于 **Axum** 的 REST API,提供数据源管理、定时数据传输、SQL 查询、看板、HTTP API 开放与 AI 对话能力。Monorepo 总览见[上级 README](../README.md)。

## ✨ 特性

- 🔐 **JWT 认证** — 登录签发 access/refresh 双令牌,cookie + Bearer 双通道,`/api/refreshToken` 刷新
- 🗄️ **多数据库元数据底座** — MySQL / PostgreSQL / SQLite 三方言可配(`[db].db_type`),基于 sea-orm
- 🐬 **DuckDB 分析引擎** — 链接官方预编译 libduckdb,SQL 查询与看板的分析算力
- 🔄 **传输任务** — Cron 定时把外部数据源数据同步进 DuckDB,SSE 推送任务事件,支持启停/试连/批量
- 📊 **SQL 查询** — 片段复用、语法校验、ad-hoc 即席查询、可视化配置(viz-config)
- 📈 **看板** — 看板配置与图表数据运行
- 🌐 **HTTP API 开放** — 把已保存的查询发布为受管理的 HTTP 接口(启停控制)
- 🤖 **AI 对话** — 多供应商模型配置,chat/completions SSE 流式转发
- 🔒 **安全** — 数据源凭据 AES-GCM 加密存储,全量实体软删除
- 📝 **可观测** — tracing 结构化日志(落盘 `[log].dir`)、Prometheus 指标 `/api/metrics`

## 🛠️ 技术栈

| 类别 | 选型 |
|------|------|
| Web 框架 | [Axum](https://github.com/tokio-rs/axum) 0.8 + Tokio |
| ORM | [sea-orm](https://github.com/SeaQL/sea-orm) 2.0.0-rc(sqlx rustls;本仓 vendor 内嵌,见 `../VENDOR.md`) |
| 分析库 | [DuckDB](https://duckdb.org) 1.5.x(r2d2 池;**官方预编译动态库**,禁用 bundled 自编) |
| 缓存 | Redis(deadpool-redis 连接池) |
| 认证 | jsonwebtoken(access + refresh) |
| 调度 | tokio-cron-scheduler |
| 校验 / 错误 | validator · thiserror(`AppError` 统一错误) |
| 日志 / 指标 | tracing + tracing-appender · metrics + prometheus |
| 脚本引擎 | rhai(表达式求值) |

## 📁 项目结构

```
src/
├── main.rs                 # 入口:路由装配(/api nest)、静态文件托管、/health
├── app.rs                  # AppState = infra(Infrastructure) + services(ServiceContainer)
├── auth/                   # 登录 / 登出 / 刷新令牌
├── sys/user/               # 用户管理
├── biz/                    # 业务模块(均在 biz/mod.rs 注册)
│   ├── ai_chat/            #   AI 对话(SSE 流式)
│   ├── ai_provider/        #   模型供应商管理
│   ├── dashboard/          #   看板
│   ├── datasource/         #   数据源(连接测试、schema 探查)
│   ├── httpapi/            #   查询开放为 HTTP API
│   ├── sqlquery/           #   SQL 查询(校验 / ad-hoc / viz-config)与 SQL 片段
│   └── transtask/          #   传输任务(cron 调度、SSE 事件)
├── common/                 # 公共模型、job 框架、宏、R<T> 响应封装
├── config/                 # config.toml 加载
├── error/                  # AppError 统一错误(thiserror)
├── middleware/             # JWT 鉴权等中间件
├── util/                   # crypto(AES-GCM)、jwt、duckdb、job 等工具
└── bin/                    # init_db(建库)· diagnose_pg · duckdb_diag
db/migrations/
├── {sqlite,mysql,postgresql}/0001_init.sql   # 全新安装(cargo run --bin init_db)
└── 000N_*.sql                                # 存量库手动执行的增量脚本(三方言)
```

每个业务模块内部分层为 `handler/ → service/ → repository/`,配 `model/`(实体 + CreateBo/UpdateBo/ListVo)与 `route/`。新增模块的完整步骤见 [../AGENTS.md](../AGENTS.md)。

## 🚀 快速开始

### 环境要求

- Rust 工具链(edition 2024,需较新 stable)
- 元数据库三选一:MySQL 8+ / PostgreSQL / SQLite
- Redis
- DuckDB:**官方预编译 libduckdb**(crate 的 `bundled` feature 已被 build.rs 硬性拦截,自编库在 Windows 有 FFI 崩溃缺陷)

### 步骤

1. **准备 libduckdb**(自动下载/就位到 `../scripts/build/vendor/libduckdb-current`,`.cargo/config.toml` 已自动指向):

   ```bash
   ../scripts/dev/setup-duckdb.sh      # Windows: ../scripts/dev/setup-duckdb.ps1
   ```

2. **配置 `config.toml`**(至少 `[db]` 与 `[redis]`):

   ```toml
   [server]
   port = 3000
   host = "0.0.0.0"

   [db]
   db_type = "postgres"        # "mysql" | "postgres" | "sqlite"
   url = "localhost"
   port = 5432
   db_name = "hdata_analysis"
   username = "postgres"
   password = "..."

   [redis]
   url = "localhost"
   port = 6379

   [jwt]
   secret = "请替换为随机长密钥"
   access_ttl = 3600
   refresh_ttl = 1800000

   [encryption]
   key = "32 位十六进制密钥,用于数据源凭据 AES-GCM 加密"

   [duckdb]
   data_dir = "data"           # 传输任务每表 DuckDB 文件根目录
   ```

3. **初始化元数据库**(全新安装;存量库升级请手动执行 `db/migrations/000N_*.sql`):

   ```bash
   cargo run --bin init_db
   ```

4. **启动**:

   ```bash
   cargo run            # http://localhost:3000;热重载可用 cargo watch -x run
   ```

## 📚 API 概览

统一响应体 `{ code, message, data? }`,除登录/健康检查外均需鉴权。完整路由以 `src/**/route/` 为准,以下为代表性端点:

| 模块 | 端点示例 |
|------|----------|
| 认证 | `POST /api/login` · `POST /api/logout` · `POST /api/refreshToken` |
| 用户 | `GET /api/user/page` · `GET /api/user/info` · `PUT /api/user/password` |
| 数据源 | `GET /api/datasource/page` · `GET /api/datasource/test/{id}` · `GET /api/datasource/{id}/schema` |
| 传输任务 | `GET /api/transtask/page` · `POST /api/transtask/run/{id}` · `POST /api/transtask/start/{id}` / `stop/{id}` · `GET /api/transtask/events`(SSE) |
| SQL 片段 | `GET /api/sqlsnippet/page`(增删改查同域) |
| SQL 查询 | `GET /api/sqlquery/page` · `POST /api/sqlquery/validate` · `POST /api/sqlquery/run-adhoc` · `GET /api/sqlquery/run/{id}` · `PUT /api/sqlquery/{id}/viz-config` |
| 看板 | `GET/PUT /api/dashboard/config` · `GET /api/dashboard/run/{id}` |
| HTTP API | `GET /api/httpapi/page` · `PUT /api/httpapi/{id}/enable` / `disable` |
| AI | `GET/POST /api/ai/providers` · `POST /api/ai/providers/{id}/test` · AI 对话 SSE 流式接口 |
| 运维 | `GET /health`(免鉴权)· `GET /api/metrics`(Prometheus) |

请求示例:

```bash
# 登录取 token
curl -X POST http://localhost:3000/api/login \
  -H "Content-Type: application/json" \
  -d '{"username": "admin", "password": "..."}'

# 携带 token 获取当前用户信息
curl http://localhost:3000/api/user/info \
  -H "Authorization: Bearer <token>"
```

## 🔧 开发命令

```bash
cargo run                    # 开发服务器(需 config.toml + 元数据库 + Redis)
cargo build --release        # 生产构建
cargo fmt                    # 格式化(rustfmt.toml:max_width=80, edition=2024)
cargo clippy                 # Lint
cargo test                   # 全量测试(~87 个,DB 相关走 feature gate 或 mock)
cargo test <name>            # 单个测试
cargo watch -x run           # 热重载(需 cargo-watch)
```

## 🏗️ 架构要点

- **AppState 双容器**:`infra`(DatabaseConnection、Redis 池、调度器、JWT/加密配置、DuckDB 数据目录、broadcast Sender)+ `services`(按域构建的服务实例,构造顺序即依赖顺序)。
- **统一错误**:`AppError`(thiserror)→ HTTP 状态码 + `R<T>` 响应体,服务层 `?` 透传,**禁止 panic**。
- **软删除**:实体统一带 `BaseEntity` 审计字段,删除置 `deleted_by/deleted_date`,物理删除仅显式 true-delete 方法。
- **静态托管**:`[server].static_dir` 托管前端构建产物,SPA 路由 fallback 到 `index.html`。

## 📦 构建与部署

- 构建脚本:`../scripts/build/`(build-windows.ps1 / build-linux.sh / build-docker.sh)
- Windows 裸机部署清单(libduckdb.dll、离线 extensions、版本耦合):**[DEPLOY.md](DEPLOY.md)**
- Docker:`../docker/backend.Dockerfile` + `docker-compose.yml.template`

## 📄 许可证

见 [LICENSE](LICENSE)。
