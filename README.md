# TinyAxum

**Axum + SvelteKit 应用骨架**。从 `hdata-analysis` 剥离全部数据分析业务后得到，
保留认证、用户管理、CRUD 前端体系，以及配置 / 错误 / 日志 / 调度等基础设施，
作为 HMSG 重构（SQLite 缓冲 + 批量同步 MySQL）的底座。

## 目录结构

```
tinyaxum/
├── tinyaxum-rs/     # Axum REST API 后端（Rust 2024 · sea-orm 2.0）
├── tinyaxum-ui/     # SvelteKit 前端（Svelte 5 · Tailwind v4 · shadcn-svelte）
├── docker/          # 后端运行时镜像与 compose 模板
└── .gitignore
```

## 快速开始

前置：Rust（edition 2024）、Node 20+、pnpm。

### 后端

```bash
cd tinyaxum-rs
cargo run --bin init_db   # 在 data/tinyaxum.db 建表并播种 admin/admin123
cargo run                 # http://127.0.0.1:3001
```

- 默认元数据库为 SQLite（`config.toml` 的 `[db].db_name = "data/tinyaxum.db"`）。
- 也支持 MySQL / PostgreSQL：改 `[db].db_type` 及连接字段后重新执行 `init_db`。
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

登录用例管理（默认账号 **admin / admin123**，首次登录后请修改密码）。

## 当前保留的能力

- JWT 认证（access/refresh 双令牌，cookie + Bearer）
- 用户管理（分页、软删除、密码修改）
- 设置页（个人资料 / 账户 / 外观 / 通知 / 显示）
- 前端 CRUD 体系（`useCrudTable`、`CrudTableShell`、`FormDrawer`、`ConfirmDialog`）
- 配置向导（`--write-config` / `--check-config`）
- 三方言数据库支持（SQLite / MySQL / PostgreSQL）
- 日志、Prometheus 指标（`/api/metrics`）、JobScheduler、静态文件托管

已剥离：DuckDB、Redis、AI、dashboard、sqlquery、datasource、httpapi、transtask
及其前端页面与依赖。

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

见 `tinyaxum-rs/LICENSE`。
