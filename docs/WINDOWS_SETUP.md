# Windows 开发环境安装指南

本文从一台未配置 Tauri 环境的 Windows 10/11 电脑开始，所有项目命令均使用 pnpm。

## 1. 安装 Node.js

建议安装 Node.js 22 LTS。可从 [Node.js 官网](https://nodejs.org/)下载安装，或在 PowerShell 执行：

```powershell
winget install OpenJS.NodeJS.LTS
```

关闭并重新打开 PowerShell，然后确认：

```powershell
node --version
npm --version
```

## 2. 启用 pnpm

Node.js 自带 Corepack，可用它安装并固定 pnpm：

```powershell
corepack enable
corepack prepare pnpm@10.11.1 --activate
pnpm --version
```

如果系统提示没有 Corepack，也可以执行：

```powershell
npm install --global corepack
corepack enable
corepack prepare pnpm@10.11.1 --activate
```

## 3. 安装 Microsoft C++ 构建工具

Tauri 在 Windows 上需要 MSVC 编译器和 Windows SDK。

### 图形界面安装

1. 下载并运行 [Visual Studio Build Tools 2022](https://visualstudio.microsoft.com/visual-cpp-build-tools/)。
2. 勾选“使用 C++ 的桌面开发”。
3. 在右侧确认包含：
   - MSVC v143 C++ x64/x86 生成工具
   - Windows 10 SDK 或 Windows 11 SDK
   - C++ CMake tools for Windows（建议）
4. 点击安装，完成后重启终端；如果安装器要求重启 Windows，请先重启。

### winget 安装

```powershell
winget install Microsoft.VisualStudio.2022.BuildTools `
  --override "--wait --passive --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
```

## 4. 安装 Rust

Tauri 2 使用 Rust 编译桌面后端。推荐通过 rustup 管理工具链：

```powershell
winget install Rustlang.Rustup
```

也可以访问 [rustup.rs](https://rustup.rs/) 下载 `rustup-init.exe`。安装程序询问时选择默认选项：

```text
1) Proceed with standard installation
```

安装后关闭并重新打开 PowerShell，让 `%USERPROFILE%\.cargo\bin` 加入 PATH，然后执行：

```powershell
rustup default stable-msvc
rustup update
rustc --version
cargo --version
rustup show active-toolchain
```

最后一条命令应显示类似：

```text
stable-x86_64-pc-windows-msvc (default)
```

如果仍提示找不到 `cargo`，可在当前 PowerShell 临时添加路径：

```powershell
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
```

然后重新执行版本检查。

## 5. 确认 WebView2

Windows 11 通常已内置 WebView2。Windows 10 或精简系统若缺少它，请安装：

```powershell
winget install Microsoft.EdgeWebView2Runtime
```

WebView2 是 Tauri 显示 React 界面的系统运行时。

## 6. 安装项目依赖

进入项目目录：

```powershell
cd D:\code\test\tauriTest
pnpm install
```

pnpm 10 如果提示依赖构建脚本被忽略，可执行：

```powershell
pnpm approve-builds
```

在交互列表中允许 Prisma、`@prisma/engines`、`@prisma/client` 和 `esbuild`，然后重新执行：

```powershell
pnpm install
```

## 7. 初始化 Prisma 开发数据库

```powershell
pnpm db:generate
pnpm db:deploy
```

这会读取 `.env` 和 `prisma/schema.prisma`，并创建 `prisma/dev.db`。桌面应用使用独立的用户数据库，
但两者共享 Prisma 迁移文件。`db:deploy` 脚本会先确保 SQLite 文件存在，以兼容部分 Windows 环境下
Prisma 迁移引擎无法自动创建空数据库文件的问题。

## 8. 启动开发环境

```powershell
pnpm tauri:dev
```

该命令会同时启动 Vite 和 Tauri 窗口。首次运行需要下载并编译 Rust 依赖，等待时间通常比之后更长。

应用启动后可以完成：

- 查看列表
- 新增记录
- 编辑记录
- 删除记录
- 关闭并重新打开应用后继续读取原有数据

## 9. 构建 Windows 安装包

先执行代码检查：

```powershell
pnpm lint
pnpm build
```

再构建正式安装包：

```powershell
pnpm tauri:build
```

输出目录：

```text
src-tauri\target\release\bundle\
```

本项目默认生成 `nsis\` 目录下的 Windows NSIS 安装程序。选择 NSIS 是为了避免部分 Windows
环境中 WiX/MSI 对中文路径和本地化元数据处理不一致的问题。

安装后的应用不需要 Node.js、pnpm、Rust 或 Prisma 环境。

## 10. 常见问题

### `cargo` 或 `rustc` 不是命令

关闭所有终端重新打开，或临时执行：

```powershell
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
```

### 找不到 `link.exe` 或 Windows SDK

重新打开 Visual Studio Installer，修改 Build Tools，确认“使用 C++ 的桌面开发”、MSVC v143 和 Windows SDK 已安装。

### Vite 端口 1420 被占用

结束占用 1420 端口的程序。项目启用了 `strictPort`，避免 Tauri 打开错误的开发地址。

### Prisma 下载引擎失败

确认网络可访问 npm 与 Prisma 下载地址，然后执行：

```powershell
pnpm store prune
pnpm install
pnpm db:generate
```

### 安装包构建很慢

首次正式构建需要编译全部 Rust Release 依赖并下载打包工具，这是正常现象。后续构建会复用缓存。
