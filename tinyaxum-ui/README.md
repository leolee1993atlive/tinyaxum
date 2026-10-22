# tinyaxum-ui

TinyAxum 的 Web 前端：登录、用户管理与设置页，配套 CRUD 表格 / 抽屉 / 确认框体系。
Monorepo 总览见[上级 README](../README.md)。

## 技术栈

| 类别 | 选型                                                                            |
| ---- | ------------------------------------------------------------------------------- |
| 框架 | [SvelteKit](https://kit.svelte.dev) 3 · [Svelte](https://svelte.dev) 5（runes） |
| 语言 | TypeScript（strict）                                                            |
| 样式 | Tailwind CSS v4 · shadcn-svelte（bits-ui）                                      |
| 表格 | `@tanstack/table-core`（服务端分页/排序）                                       |
| 校验 | zod                                                                             |
| 构建 | Vite 8 · `@sveltejs/adapter-static`（SPA，`index.html` fallback） |

## 快速开始

前置：Node 20+、pnpm；后端需在本机 3001 端口运行（见 [tinyaxum-rs/README](../README.md#后端)）。

```bash
pnpm install
pnpm dev          # http://localhost:5173，/api 代理到 http://127.0.0.1:3001
```

默认账号：**admin / admin123**。

## 脚本

| 命令                                | 说明                                    |
| ----------------------------------- | --------------------------------------- |
| `pnpm dev`                          | 开发服务器（`/api` 代理到 3001）        |
| `pnpm build`                        | 生产构建（adapter-static → `build/`）   |
| `pnpm preview`                      | 预览生产构建                            |
| `pnpm check`                        | svelte-check 类型检查（strict TS）      |
| `pnpm format` / `pnpm format:check` | Prettier 写入 / 检查                    |
| `pnpm lint`                         | Prettier 检查（唯一 linter，无 ESLint） |

## 项目结构

```
src/
├── routes/
│   ├── (app)/           # 认证后的主应用壳
│   │   ├── users/       #   用户管理
│   │   └── settings/    #   设置（账户/外观/显示/通知）
│   ├── (auth)/          # 登录页
│   └── (errors)/        # 401 / 403 / 404 / 500 / 503
└── lib/
    ├── api/             # client / auth / users / types
    ├── components/
    │   ├── ui/          # shadcn-svelte 基础组件
    │   ├── crud/        # CrudTableShell
    │   ├── data-table/  # 表格配套组件
    │   └── layout/      # 侧边栏、Header、Main
    ├── composables/     # useCrudTable 等
    └── utils.ts         # cn()（twMerge + clsx）等
```

## 与后端的衔接

- **认证**：cookie（`credentials: 'include'`），401 时 `ApiClient` 自动队列挂起请求、
  刷新令牌后重放。
- **部署**：`pnpm build` 产物是纯静态 SPA，交由后端 `[server].static_dir` 托管，
  由后端做 SPA fallback，单端口对外；开发期才走 Vite 代理。

## 工程约定

- Svelte 5 runes 风格：组件用 `$props()` 解构，不用 `export let`；`#lib` 导入带 `.js` 后缀。
- 列表页统一走 `useCrudTable`；表单新建/编辑用 `FormDrawer` 抽屉，删除确认用 `ConfirmDialog`。
- 格式化只有 Prettier（无分号、单引号、2 空格、80 列）；提交前跑
  `pnpm format:check && pnpm check`。

更完整的架构与代码风格约定见 [../AGENTS.md](../AGENTS.md)。
