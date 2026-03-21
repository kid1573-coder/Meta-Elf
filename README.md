# 元精灵

Windows 桌面悬浮看盘：透明置顶主窗口、自选股与可配置列表列、设置页（字体/极简模式/贴边隐藏/涨跌色/老板键等）、系统托盘与 **Ctrl+Shift+H**（可在设置中修改）切换主窗口显示。

技术栈：**Tauri 2 + Vue 3 + Vite**。行情当前为 **Rust 侧 Mock**，通过 `src-tauri/src/quotes.rs` 中的 `QuoteProvider` 可替换为真实数据源。

## 环境要求

- [Node.js](https://nodejs.org/)（建议 LTS）
- [Rust](https://www.rust-lang.org/tools/install) + Windows 端 **Microsoft C++ Build Tools**（见 [Tauri 前置条件](https://tauri.app/start/prerequisites/)）

## 开发

```bash
npm install
npm run tauri dev
```

### Cursor / VS Code 提示 `cargo` / `program not found`

若已安装 Rust，但终端里仍报 `failed to run 'cargo metadata' ... program not found`：

1. **完全退出并重新打开 Cursor**（或至少关掉所有终端面板再开新终端），以便继承最新的用户 `Path`。
2. 本项目已在 [`.vscode/settings.json`](.vscode/settings.json) 中为集成终端把 `%USERPROFILE%\.cargo\bin` 加到 `Path` 前面；新开终端后应能直接找到 `cargo`。
3. 临时验证：在 PowerShell 中执行  
   `$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"`  
   再运行 `cargo --version`、`npm run tauri dev`。

## 构建与分发

```bash
npm run tauri build
```

安装包默认使用 **NSIS**（`src-tauri/tauri.conf.json` 中 `bundle.targets`）。构建完成后，可在 `src-tauri/target/release/bundle/` 下找到安装程序；将 **`target/release/`** 中可执行文件连同依赖一起打包为 zip，也可作为免安装目录分发（与「单文件绿色版」等效，由你方选择打包脚本）。

重新生成图标：

```bash
npm run tauri:icon
```

## 配置存储

设置保存在 `%APPDATA%\yuanjingling\settings.json`（由 `dirs` 解析的配置目录下的 `yuanjingling` 文件夹）。

## 项目结构摘要

| 路径 | 说明 |
|------|------|
| `src/views/MainPanel.vue` | 悬浮主界面、列表与工具栏 |
| `src/views/SettingsView.vue` | 设置布局与选项 |
| `src/composables/useEdgeHide.ts` | 贴边自动隐藏 |
| `src-tauri/src/lib.rs` | 托盘、老板键、窗口属性、设置读写 |
| `src-tauri/src/quotes.rs` | Mock 行情与 `QuoteProvider` 扩展点 |
