# TinyAxum

一个开箱即用的全栈应用骨架：Rust + Axum 提供 REST API，SvelteKit 提供管理界面。
内置 JWT 认证、用户管理、前端 CRUD 体系，以及配置、错误处理、日志、任务调度、
指标等基础设施。

## 技术栈

| 层 | 组成 |
| --- | --- |
| 后端 | Rust 2024 · Axum 0.8 · Tokio · sea-orm 2.0 · tokio-cron-scheduler · JWT |
| 数据库 | SQLite（默认）/ MySQL / PostgreSQL |
| 前端 | SvelteKit 3 · Svelte 5 (runes) · TypeScript · Tailwind CSS v4 · shadcn-svelte · TanStack Table v9 |

## 目录结构

```
tinyaxum/
├── tinyaxum-rs/     # Axum REST API 后端
├── tinyaxum-ui/     # SvelteKit 前端
├── docker/          # 后端运行时镜像与 compose 模板
├── LICENSE
└── README.md
```

## 快速开始

前置：Rust（edition 2024）、Node 20+、pnpm。

### 后端

```bash
cd tinyaxum-rs
cp config.example.toml config.toml   # 按需修改 [jwt].secret / [encryption].key
cargo run --bin init_db   # 在 data/tinyaxum.db 建表并创建 admin/admin123
cargo run                 # http://127.0.0.1:3001
```

- `config.toml` 为本地配置，已被 `.gitignore` 忽略；仓库只提交
  `config.example.toml`，不要提交真实密钥。
- 默认元数据库为 SQLite（`[db].db_name = "data/tinyaxum.db"`）。
- 切换 MySQL / PostgreSQL：修改 `[db].db_type` 及连接字段后重新执行 `init_db`。
- Schema 初始化是显式的，没有运行时自动迁移；已有库的变更按编号脚本手工执行。
- 健康检查：`GET /api/health`；登录：`POST /api/login`。

### 前端

```bash
cd tinyaxum-ui
pnpm install
pnpm dev      # http://localhost:5173，/api 代理到 127.0.0.1:3001
```

生产构建产物由后端托管：

```bash
pnpm build
rm -rf ../tinyaxum-rs/static && cp -r build ../tinyaxum-rs/static
```

默认账号 **admin / admin123**，首次登录后请修改密码。

## 功能

- JWT 认证：access / refresh 双令牌，支持 cookie 与 Bearer 两种传递方式
- 用户管理：分页、软删除、密码修改
- 设置页：个人资料、账户、外观、通知、显示
- 前端 CRUD 体系：`useCrudTable`、`CrudTableShell`、`FormDrawer`、`ConfirmDialog`
- 配置向导：`--write-config` / `--check-config`
- 三方言数据库支持：SQLite / MySQL / PostgreSQL
- 日志、Prometheus 指标（`/api/metrics`）、JobScheduler、静态文件托管
- 全局软删除：实体统一记录 `deleted_by` / `deleted_date`
- 敏感字段加密存储：AES-GCM

## 常用命令

### 后端（`tinyaxum-rs/`）

```bash
cargo fmt
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
cargo test
```

### 前端（`tinyaxum-ui/`）

```bash
pnpm format
pnpm check
pnpm build
```

## 许可证

[Apache License 2.0](LICENSE)
