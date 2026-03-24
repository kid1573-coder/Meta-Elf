# 元精灵

Windows 桌面悬浮看盘：透明置顶主窗口、自选股与可配置列表列、设置页（字体/极简模式/贴边隐藏/涨跌色/老板键等）、系统托盘与 **Ctrl+Shift+H**（可在设置中修改）切换主窗口显示。

技术栈：**Tauri 2 + Vue 3 + Vite**。行情默认走 **东方财富公开 HTTP 接口**（Rust `reqwest` 在 `src-tauri/src/quotes.rs` 中拉取并映射为统一 `QuoteRow`）；可在设置中切换为 **本地 Mock** 做离线调试。公开接口不保证长期稳定，可能有频率限制，仅供个人盯盘参考，不构成投资建议。

## 环境要求

- [Node.js](https://nodejs.org/)（建议 LTS）
- [Rust](https://www.rust-lang.org/tools/install) + Windows 端 **Microsoft C++ Build Tools**（见 [Tauri 前置条件](https://tauri.app/start/prerequisites/)）

## 开发

```bash
npm install
npm run tauri dev
```

开发时 Vite 使用端口 **1430**（与 `devUrl` 一致）。`npm run dev` / `tauri dev` 会先通过 **kill-port** 释放该端口，避免上次未退干净的 Vite 占用导致 `beforeDevCommand` 失败。若仍冲突，可手动：`Get-NetTCPConnection -LocalPort 1430` → `Stop-Process -Id <PID> -Force`；或改 `vite.config.ts` 与 `src-tauri/tauri.conf.json` 中的端口并保持两处相同。

Windows 下若终端找不到 `cargo`，也可执行：

```bash
npm run tauri:dev
```

（会调用 `scripts/tauri-dev.ps1` 把 `%USERPROFILE%\.cargo\bin` 临时加入 `Path`。）

### Cursor / VS Code 提示 `cargo` / `program not found`

若已安装 Rust，但终端里仍报 `failed to run 'cargo metadata' ... program not found`：

1. **完全退出并重新打开 Cursor**（或至少关掉所有终端面板再开新终端），以便继承最新的用户 `Path`。
2. 本项目已在 [`.vscode/settings.json`](.vscode/settings.json) 中为集成终端把 `%USERPROFILE%\.cargo\bin` 加到 `Path` 前面；新开终端后应能直接找到 `cargo`。
3. 临时验证：在 PowerShell 中执行  
   `$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"`  
   再运行 `cargo --version`、`npm run tauri dev`（或 `npm run tauri:dev`）。

## 构建与分发

```bash
npm run tauri build
```

安装包默认使用 **NSIS**（`src-tauri/tauri.conf.json` 中 `bundle.targets`）。构建完成后，可在 `src-tauri/target/release/bundle/` 下找到安装程序；将 **`target/release/`** 中可执行文件连同依赖一起打包为 zip，也可作为免安装目录分发（与「单文件绿色版」等效，由你方选择打包脚本）。

应用图标源文件为 [`src-tauri/icon-square.png`](src-tauri/icon-square.png)（1024×1024，精灵主题）。若替换设计，覆盖该文件后执行：

```bash
npx tauri icon src-tauri/icon-square.png
```

或使用脚本（会先跑占位 PNG 生成器再出图，一般改用上一行即可）：

```bash
npm run tauri:icon
```

重新打包或重启 `tauri dev` 后，任务栏与托盘会使用新图标；若系统仍显示旧块，可结束应用后重开，或注销/重启一次刷新图标缓存。

## 配置存储

设置保存在 `%APPDATA%\yuanjingling\settings.json`（由 `dirs` 解析的配置目录下的 `yuanjingling` 文件夹）。

## 项目结构摘要

| 路径 | 说明 |
|------|------|
| `src/views/MainPanel.vue` | 悬浮主界面、列表与工具栏 |
| `src/views/SettingsView.vue` | 设置布局与选项 |
| `src/composables/useEdgeHide.ts` | 贴边自动隐藏 |
| `src-tauri/src/lib.rs` | 托盘、老板键、窗口属性、设置读写 |
| `src-tauri/src/quotes.rs` | 东财 HTTP 行情、Mock、搜索联想与 `QuoteProvider` |
| `src/views/WatchlistAddView.vue` | 添加自选 |
| `src/views/StockDetailView.vue` | 个股详情 |

## 界面规范与优化清单

与主看盘面板统一的字体、色板、**输入/下拉（`.yj-field-control`）** 与 **弹窗浮层（`--yj-modal-*`，勿用表格区半透明底）** 说明见 **[docs/UI_CONSISTENCY.md](docs/UI_CONSISTENCY.md)**。  
各页面改造进度与历史记录见 **[docs/UI_OPTIMIZATION_CHECKLIST.md](docs/UI_OPTIMIZATION_CHECKLIST.md)**。  
财联社类「24 小时滚动新闻」**能否接入、合规边界与选型**见 **[docs/NEWS_DATA_SOURCES.md](docs/NEWS_DATA_SOURCES.md)**（说明文档，非已实现功能）。
