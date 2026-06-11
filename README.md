# Tauri React Prisma CRUD

一个本地桌面列表管理示例，支持新增、查询、编辑和删除，数据持久化到 SQLite。

## 技术栈

- Tauri 2
- React 19 + TypeScript + Vite
- Ant Design 6
- Prisma 6（数据模型与迁移）
- SQLite + Rust `rusqlite`（桌面运行时）
- pnpm

> Prisma Client 是 Node.js 客户端，不能直接运行在 Tauri 的 Rust 进程中。本项目使用 Prisma 管理
> Schema 和迁移 SQL，Tauri 运行时通过 Rust 读取同一套迁移并访问 SQLite。安装后的应用不要求用户安装 Node.js。

## 快速开始

首次使用请先阅读 [Windows 环境安装指南](docs/WINDOWS_SETUP.md)。

```powershell
pnpm install
pnpm db:generate
pnpm db:deploy
pnpm tauri:dev
```

构建 Windows 安装包：

```powershell
pnpm tauri:build
```

构建产物位于：

```text
src-tauri/target/release/bundle/
```

## 数据位置

- Prisma 开发数据库：`prisma/dev.db`
- 桌面应用数据库：Windows 应用数据目录下的 `items.db`

桌面数据库在应用首次启动时自动创建，表结构来自
`prisma/migrations/20260611000000_init/migration.sql`。

## 常用命令

| 命令 | 用途 |
| --- | --- |
| `pnpm tauri:dev` | 启动完整桌面开发环境 |
| `pnpm dev` | 只启动前端网页，不具备 Tauri 数据命令 |
| `pnpm build` | 检查 TypeScript 并构建前端 |
| `pnpm lint` | 检查前端代码 |
| `pnpm db:generate` | 根据 Prisma Schema 生成客户端 |
| `pnpm db:migrate` | 开发阶段创建新的 Prisma 迁移 |
| `pnpm db:deploy` | 将已有迁移应用到开发数据库 |
| `pnpm tauri:build` | 构建 exe 和 Windows 安装包 |

## 新增数据库字段

1. 修改 `prisma/schema.prisma`。
2. 执行 `pnpm db:migrate --name 字段名称`。
3. 检查新生成的 `migration.sql`。
4. 在 `src-tauri/src/lib.rs` 中同步 Rust 数据结构和 SQL。
5. 执行 `pnpm tauri:dev` 验证升级后的已有数据库。
