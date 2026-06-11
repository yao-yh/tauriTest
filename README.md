# ListNest

ListNest 是一个基于 Tauri 2、React 19、Ant Design、Prisma 和 SQLite 的本地清单管理示例。
它包含完整 CRUD、数据库版本迁移，以及安装包内置资源首次落盘能力。

## 应用标识

- 显示名称：`ListNest`
- 窗口标题：`ListNest 本地清单`
- Windows 进程：`listnest.exe`
- 安装包：`ListNest_<版本>_x64-setup.exe`
- 应用标识：`com.example.tauri-react-prisma-crud`
- 图标源文件：`src-tauri/icons/app-icon-source.png`

应用标识故意保持不变，以便从旧版升级时继续使用原安装记录和用户数据目录。

`pnpm tauri icon` 会生成 Windows、macOS、iOS 和 Android 的完整尺寸集。当前 Windows
打包配置只直接引用 PNG、ICO 和 ICNS 中的核心文件；其余尺寸可为未来跨平台发布保留。

## 快速开始

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

产物位于 `src-tauri/target/release/bundle/nsis/`。

## 数据库升级

Prisma 负责维护 `prisma/schema.prisma` 和迁移 SQL。桌面应用启动时读取
`src-tauri/src/lib.rs` 中的 `MIGRATIONS` 列表，并通过用户数据库中的
`_app_migrations` 表确保每个迁移只执行一次。

新增数据库变更时：

1. 修改 `prisma/schema.prisma`。
2. 执行 `pnpm db:migrate --name 迁移名称`。
3. 将新迁移通过 `include_str!` 加入 Rust 的 `MIGRATIONS` 列表。
4. 同步 Rust 数据结构和查询。
5. 使用旧版本数据库验证升级。

迁移必须向前兼容，发布后不能修改已经执行过的迁移文件，只能新增迁移。

## 安装包内置资源

示例资源位于 `src-tauri/resources/starter-pack/`：

- CSV 导入模板
- 默认列配置
- 离线说明文件

这些文件会进入安装包，并在应用启动时复制到：

```text
%APPDATA%\com.example.tauri-react-prisma-crud\resources\starter-pack\
```

升级策略是“只复制缺失文件，不覆盖已有同名文件”。因此用户可以编辑本地模板或配置，
新版本仍可补充新的资源文件。资源版本记录在 `app_resource_bundles` 表中。

详细升级与资源设计见 [升级与资源方案](docs/UPGRADE_AND_RESOURCES.md)。
客户端自动更新和发布流程见 [自动更新文档](docs/AUTO_UPDATE.md)。
完整环境安装步骤见 [Windows 环境安装指南](docs/WINDOWS_SETUP.md)。
