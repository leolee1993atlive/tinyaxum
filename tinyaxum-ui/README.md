# hdata-analysis-ui

hdata-analysis 平台的 Web 前端:数据源管理、传输任务、SQL 查询(集成 CodeMirror 编辑器)、可视化看板与 AI 对话的单页应用。Monorepo 总览见[上级 README](../README.md)。

## 技术栈

| 类别       | 选型                                                                                                            |
| ---------- | --------------------------------------------------------------------------------------------------------------- |
| 框架       | [SvelteKit](https://kit.svelte.dev) 2 · [Svelte](https://svelte.dev) 5(runes:`$state` / `$props` / `$bindable`) |
| 语言       | TypeScript(strict)                                                                                              |
| 样式       | Tailwind CSS v4 · shadcn-svelte(bits-ui)                                                                        |
| SQL 编辑器 | CodeMirror 6(`@codemirror/lang-sql`、lint、one-dark)                                                            |
| 图表       | D3 全家桶 · layerchart                                                                                          |
| 表格       | `@tanstack/table-core`(服务端分页/排序)                                                                         |
| 其他       | zod 校验 · sql-formatter · marked + DOMPurify(MD 渲染)                                                          |
| 构建       | Vite 7 · `@sveltejs/adapter-static`(SPA,`index.html` fallback)                                                  |

## 快速开始

前置:Node 20+、pnpm;需要后端已在本机 3000 端口运行(见 [hdata-analysis-rs/README.md](../hdata-analysis-rs/README.md))。

```bash
pnpm install
pnpm dev          # http://localhost:5173,/api 代理到 http://localhost:3000
```

## 脚本

| 命令                                | 说明                                               |
| ----------------------------------- | -------------------------------------------------- |
| `pnpm dev`                          | 开发服务器(Vite,`/api` 代理到 localhost:3000)      |
| `pnpm build`                        | 生产构建(adapter-static,SPA fallback `index.html`) |
| `pnpm preview`                      | 预览生产构建                                       |
| `pnpm check`                        | svelte-check 类型检查(strict TS)                   |
| `pnpm check:watch`                  | 类型检查 watch 模式                                |
| `pnpm format` / `pnpm format:check` | Prettier 写入 / 检查                               |
| `pnpm lint`                         | Prettier 检查(唯一 linter,无 ESLint)               |

## 项目结构

```
src/
├── routes/
│   ├── (app)/           # 认证后的主应用壳
│   │   ├── dashboard/   #   看板
│   │   ├── datasource/  #   数据源
│   │   ├── tasks/       #   传输任务
│   │   ├── sqlquery/    #   SQL 查询
│   │   ├── sqlsnippet/  #   SQL 片段
│   │   ├── httpapi/     #   HTTP API 管理
│   │   ├── ai-provider/ #   模型供应商
│   │   ├── chats/       #   AI 对话
│   │   ├── users/       #   用户管理
│   │   └── settings/    #   设置
│   ├── (auth)/          # 登录等认证页
│   └── (errors)/        # 401 / 403 / 404 / 500 / 503
└── lib/
    ├── api/             # 按域拆分的 API 模块(datasources、transtask、dashboard…)
    │   └── client.ts    #   ApiClient 单例:请求排队,401 后刷新重试
    ├── components/
    │   ├── ui/          # shadcn-svelte 基础组件
    │   └── …            # crud、data-table、charts、sql-editor 等业务组件
    ├── composables/     # useCrudTable 等组合式函数
    └── utils.ts         # cn()(twMerge + clsx)等
```

## 与后端的衔接

- **认证**:cookie(`credentials: 'include'`),401 时 `ApiClient` 自动队列挂起请求、刷新令牌后重放。
- **部署**:`pnpm build` 产物是纯静态 SPA,交由后端 `[server].static_dir` 托管,由后端做 SPA fallback,单端口对外;开发期才走 Vite 代理。

## 工程约定

- Svelte 5 runes 风格:组件用 `$props()` 解构,不用 `export let`;`$lib` 导入带 `.js` 后缀。
- 列表页统一走 `useCrudTable`(TanStack Table 服务端分页排序);表单新建/编辑用 `FormDrawer` 抽屉。
- 格式化只有 Prettier(无分号、单引号、2 空格、80 列);提交前跑 `pnpm format:check && pnpm check`。

更完整的架构与代码风格约定见 [../AGENTS.md](../AGENTS.md)。
