# AGENTS.md — TinyAxum

Monorepo with two workspaces:
- **`tinyaxum-rs/`** — Axum REST API backend (Rust 2024)
- **`tinyaxum-ui/`** — SvelteKit web frontend

This repository is an application skeleton extracted from `hdata-analysis`: authentication,
user management, the CRUD frontend system, and the config/error/log/scheduler
infrastructure remain. Analytics business modules (DuckDB, Redis, AI, dashboard,
sqlquery, datasource, httpapi, transtask) were removed.

## Build / Lint / Test Commands

### Rust Backend (`tinyaxum-rs/`)

```bash
cargo run                          # Dev server (requires config.toml + DB)
cargo run --bin init_db            # Create schema + seed admin/admin123
cargo build --release              # Production build
cargo fmt                          # Format (rustfmt.toml: max_width=80, edition=2024)
cargo check --all-targets          # Type check
cargo clippy --all-targets -- -D warnings
cargo test                         # Run all tests
cargo test test_name               # Run a single test
```

Tests use temporary directories / in-memory config and do not require a running database.

### SvelteKit Frontend (`tinyaxum-ui/`)

```bash
pnpm install                       # Install deps
pnpm dev                           # Dev server (Vite; proxies /api → 127.0.0.1:3001)
pnpm build                         # Production build (adapter-static → build/)
pnpm check                         # Type-check (svelte-check, strict TS)
pnpm format                        # Format with Prettier
pnpm format:check                  # Check formatting
pnpm lint                          # Prettier check (no ESLint)
```

## Architecture

### Rust Backend
- **Stack**: Rust 2024 · Axum 0.8 · Tokio · sea-orm 2.0 · tokio-cron-scheduler · JWT
- **Database support**: MySQL, PostgreSQL, or SQLite (`[db].db_type`). Default is SQLite
  at `data/tinyaxum.db`. Schema init is manual: `src/bin/init_db.rs` executes
  `db/migrations/<db_type>/0001_init.sql` and seeds `admin/admin123`.
  Existing installs apply numbered incremental scripts by hand; there is no runtime
  auto-migration.
- **Layered**: `handler/` → `service/` → `repository/` per module, with `model/` and `route/`.
- **Active modules**: `auth/`, `sys/user`. Add new business modules under `src/biz/<module>/`
  and register them in `src/biz/mod.rs` (re-create the barrel when the first module lands).
- **AppState** (DI container): `AppState.infra: Infrastructure` holds `DatabaseConnection`,
  `JobScheduler`, JWT config, and encryption key. `AppState.services: ServiceContainer`
  holds one service instance per domain, constructed from `infra`.
- **Auth**: JWT Bearer + cookie. Login at `/api/login`, refresh at `/api/refreshToken`,
  middleware validates protected routes.
- **Metrics**: `metrics` + `metrics-exporter-prometheus`; served at `/api/metrics`.
- **Static serving**: `config.server.static_dir` hosts the built frontend with SPA fallback.

### SvelteKit Frontend
- **Stack**: SvelteKit 3 · Svelte 5 (runes) · TypeScript (strict) · Tailwind CSS v4 ·
  shadcn-svelte (bits-ui) · TanStack Table v9 (`@tanstack/table-core`)
- **Svelte 5 runes**: `$props()`, `$bindable()`, `$state()`, `{@render children?.()}`
- **Path aliases**: `#lib` → `src/lib`
- **Auth**: cookie-based (`credentials: 'include'`). `ApiClient` in `#lib/api/client.ts`
  queues requests and retries after 401.
- **Routes**: `(app)/` (authenticated shell: users + settings), `(auth)/` (login),
  `(errors)/` (401/403/404/500/503).
- **CRUD page pattern**: `useCrudTable` composable wraps TanStack Table with server-side
  pagination/sorting. `CrudPageState<TRow, TDialog>` tracks dialog state.
- **TanStack Table v9**: register features once in `#lib/table-features.ts` via
  `tableFeatures({...})`, then build instances with
  `constructTable({ features: appTableFeatures, ... })`. Types flow through `AppTable<T>` /
  `AppTableFeatures`; do not use v8 `createTable` or `get*RowModel` options.
- **API modules**: export object literals (`usersApi = { list, create, ... }`) using the
  singleton `apiClient`.
- **UI components**: `src/lib/components/ui/` (shadcn-svelte primitives),
  `src/lib/components/` (app components)
- **cn() utility** (`#lib/utils.ts`): `twMerge(clsx(...))` for Tailwind class merging

## Code Style — Rust

### Imports (enforced by rustfmt.toml)
Grouped automatically: `std` → external crates → internal (`crate::`). Granularity = `Crate`.

```rust
use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
};
use sea_orm::DatabaseConnection;
use validator::Validate;

use crate::{app::AppState, common::vo::response::R};
```

### Handler Pattern
- Handlers are free async functions receiving `State<Arc<AppState>>`, extractors, and
  returning `Result<R<T>, AppError>`.
- Validate input with `validator::Validate`. Use `?` to propagate `AppError`.
- Call `state.services.<module>_service.<method>().await?`.
  Return `Ok(R::ok())` / `Ok(R::ok_with_data(...))`.

### Model Pattern
- Entity structs include `#[serde(flatten)] pub base_entity: BaseEntity` for audit fields.
- Three companion types per entity: `EntityCreateBo` (input), `EntityUpdateBo` (input),
  `EntityListVo` (output).
- Bo/Vo types use `#[serde(rename_all = "camelCase")]` for JSON camelCase API fields.
- Bo types derive `Validate` with `#[validate(...)]` annotations.
- sea-orm entity files inline BaseEntity fields (not nested); implement `From<Model>` for the
  domain type and `From<&DomainType> for ActiveModel`.

### Error Handling
- **Unified `AppError` enum** in `src/error/error.rs` using `thiserror`. Variants:
  `JwtTokenError`, `DbError`, `Auth`, `Validation`, `NotFound`, `Internal`, `BadRequest`.
- `AppResult<T>` = `Result<T, AppError>`. Services return this; handlers use `?`.
- All errors implement `IntoResponse` → map to HTTP status codes + `R<T>` body.
- **Never** panic. Always propagate errors gracefully.

### Naming
- snake_case for files, functions, variables, modules.
- PascalCase for types, structs, enums.
- Module directories use snake_case; `mod.rs` re-exports submodules.

### Adding a New Backend Module
Create under `src/biz/<module>/` with subdirs: `handler/`, `service/`, `repository/`,
`model/`, `route/`. Each has a `mod.rs` barrel file. Register it in `src/biz/mod.rs`,
merge its router in `src/main.rs`, and inject the service into `ServiceContainer`
in `src/app.rs`.

### Service Construction
- Repositories take `DatabaseConnection` (from `infra.db.clone()`).
- Services take `&Infrastructure` in `new()`; cross-domain dependencies take other service
  instances.
- Construction order in `ServiceContainer::new()` matters when services depend on each other.

## Code Style — SvelteKit / TypeScript

### Prettier Config
No semicolons · single quotes · 2-space indent · 80 print width · trailing comma (es5) · LF.

### Import Order (auto-sorted by Prettier plugin)
1. `svelte`, `svelte/*`
2. `@sveltejs/*`
3. `#lib/components/ui/*`
4. `#lib/components/*`
5. `#lib/*`
6. Third-party modules
7. Relative paths (`./`, `../`)

### Component Pattern (Svelte 5)
```svelte
<script lang="ts">
  import Component from '#lib/components/...'
  import { cn, type WithElementRef } from '#lib/utils.js'

  let { ref = $bindable(null), class: className, ...restProps }: Props = $props()
</script>

<div data-slot="component-name" class={cn('base-classes', className)} {...restProps}>
  {@render children?.()}
</div>
```
- Always `<script lang="ts">`.
- Use `$props()` destructuring (not `export let`).
- Use `data-slot` attributes on wrapper elements for styling hooks.
- Use `cn()` for conditional Tailwind classes.
- Include `.js` extension on `#lib` imports: `import { cn } from '#lib/utils.js'`.

### Naming
- kebab-case for file names and component directories.
- PascalCase for component names in imports/usage.
- camelCase for TypeScript variables, functions, types.
- Route files use SvelteKit conventions: `+page.svelte`, `+layout.svelte`.

## Key Conventions

- **UI 容器三分法(前端)**: 表单新建/编辑 → `FormDrawer`(`#lib/components/FormDrawer.svelte`,
  右侧全高抽屉,单列纵向表单,宽度默认 xl/576px,`class` 覆盖;Drawer 里不要做两列网格)。
  字段块统一 `flex flex-col gap-3`(label→控件),块间距由壳的 `space-y-4` 提供。
  删除等确认交互 → `ConfirmDialog`(AlertDialog,destructive 传 `destructive`);
  单字段快输/工具弹窗 → 小 `Dialog`(直接用 `ui/dialog` 组合)。
  不要再新建居中大弹窗表单。
- **Soft delete** everywhere: entities use `BaseEntity.delete(user_id)` which sets
  `deleted_by`/`deleted_date`. Hard delete only via explicit `true_delete_*` methods.
- **Encrypted credentials**: `src/util/crypto_util.rs` provides AES-GCM encrypt/decrypt
  helpers for sensitive values.
- **API response shape**: `{ code: u16, message: string, data?: T }` via `R<T>`.
  Handlers return `Result<R<T>, AppError>`.
- **Configuration**: `config.toml` at backend root with `[server]`, `[db]`, `[jwt]`,
  `[encryption]`, `[log]`, `[pool]`, `[cors]` sections. It is local-only and gitignored;
  commit changes to `config.example.toml` instead, never real secrets. `config.sqlite.toml`
  is a local SQLite variant and is also gitignored. `[cors]` = optional Origin whitelist
  (`enabled` + `allowed_origin_patterns` with `*` wildcards); disabled by default.
- **Tracing**: handlers and service methods annotated with `#[tracing::instrument]`;
  use `tracing::info!`/`error!` inside services.
- **Static file serving**: `config.server.static_dir` hosts the built frontend; falls back
  to `index.html` for SPA routing.
- **Docker**: `docker/backend.Dockerfile` copies a host-compiled binary + static files.
  `docker/docker-compose.yml.template` with placeholder substitution.
- **Migration files**: `db/migrations/{sqlite,mysql,postgresql}/0001_init.sql` for fresh
  installs (run via `src/bin/init_db.rs`). Schema changes for existing installs go in
  numbered root-level scripts with all three dialects, applied manually.
