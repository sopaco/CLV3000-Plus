# CLV3000 Plus

面向 **Coding Agent 时代** 的 PC 实用工具（Rust + GPUI）。

## 功能

- **智能清理**：识别并清理 Rust、Node/Web、Python、Java、Android、iOS、Flutter、KMP、.NET、C/C++ 构建缓存与依赖
- **Agent 项目**：识别 Claude / Cursor / Codex / WorkBuddy 等 Agent 试验项目
- **启动项管理**：macOS LaunchAgents / 登录项，Windows 注册表与启动文件夹
- **进程管理**：按 CPU/内存查看并结束进程
- **双模式**：简单模式（非技术人员友好）/ 专家模式

## 运行

```bash
# 需要 macOS Metal Toolchain（Xcode）
# xcodebuild -downloadComponent MetalToolchain

cargo run -p clv-app
```

## 打包

### macOS（.app + .dmg）

```bash
./scripts/bundle-macos.sh
# 产物：
#   target/release/CLV3000 Plus.app
#   target/release/CLV3000 Plus.dmg
```

### Windows（带图标的 release 可执行文件）

在 Windows 上直接构建即可，`build.rs` 会通过 `embed-resource` 将 `assets/icons/icon_app.ico` 嵌入 exe：

```powershell
cargo build -p clv-app --release
# 产物：target\release\clv3000-plus.exe（含任务栏/资源管理器图标）
```
