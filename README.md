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

## 项目结构

```
crates/
  clv-core/      # 扫描规则、Agent 识别、清理执行
  clv-platform/  # 启动项、进程（macOS / Windows）
  clv-app/       # GPUI 界面
```

## 平台

- macOS 12+
- Windows 10+（启动项/进程已实现基础支持）

## 依赖

- [GPUI](https://gpui.rs) 0.2
- [gpui-component](https://github.com/longbridge/gpui-component) 0.5
