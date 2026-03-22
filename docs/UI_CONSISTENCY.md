# 界面体验一致性说明（与主面板对齐）

本文档描述「元精灵」客户端内 **字体、色板、表单、浮层与卡片** 应与主看盘面板（`MainPanel.vue`）保持一致的约定，便于后续各页对齐。

## 设计原则

1. **单一数据源**：颜色与语义以 [`src/App.vue`](../src/App.vue) 中 `.yj-root[data-theme="dark|light"]` 的 CSS 变量为准（`--yj-text`、`--yj-field-*`、`--yj-table-wrap-*` 等），子页面避免写死 `#hex` 替代语义色。
2. **字体栈统一**：全应用使用 `Noto Sans SC` + `DM Sans`（已在 `App.vue` 通过 Google Fonts 引入），正文与输入框均不应改用系统默认无衬线而不用这两套之一。
3. **字号阶梯与主面板表格对齐**（均以 `.yj-shell` 的 `font-size` 为基准，用 **`em`**）：主列表正文 **0.88em**；分组 Tab **0.78em**；表头 **0.8em**；主栏品牌名 **0.92em**。二级页 **顶栏标题约 0.86em**（不大于品牌名）、**区块小标题（卡片/设置节 h2）约 0.8em** 与表头同级；**顶栏返回/链接按钮约 0.78em**（与 Tab 一致）；**内容区 `.btn`、全局 `.modal-btn` 约 0.82em**（略大于 Tab）；说明/次要文案约 **0.78em**；**表单输入、表格正文** 仍以 **0.88em**（含 `.yj-field-control`）；设置里单选/开关等说明行可与正文同级 **0.88em**。个股详情大价 **1.55em** 左右（随「首页字体」缩放，勿用固定 `2rem` 顶在 `html` 上）。
4. **`rem` 与 `em`（重要）**：用户「首页字体」通过 `.yj-shell` 的 `font-size` 作用到 **`#yj-root`**。子树内若用 **`rem`**，基准仍是 **`html`（多为 16px）**，**不会**随该设置缩放，弹窗/菜单会比 Tab/表格显大。约定：**在 `#yj-root` 内**，与正文同级的控件、浮层、弹窗标题与按钮等字号请用 **`em`**，与主面板一致；仅在与 `#yj-root` 无关的全局层（若有）再考虑 `rem`。
5. **数字与代码**：代码、涨跌幅等使用 **`font-variant-numeric: tabular-nums`**，英文字体优先走 `DM Sans`，与主面板 `.grid td.num` 一致。

## 全局复用：表单控件 `.yj-field-control`

在 [`src/App.vue`](../src/App.vue) 中已定义：

- 类名：`yj-field-control`（用于 **`input`、`textarea`、`select`**）
- 字号：**0.88em**（勿用 `rem`，见上条），行高约 1.42，与主面板表格正文字号同级
- 颜色/背景/边框：`var(--yj-field-border)`、`var(--yj-field-bg)`、`var(--yj-field-color)`
- 圆角：**8px**（与主面板 `table-wrap`、标题栏按钮圆角体系一致）
- 占位符（input/textarea）：`var(--yj-text-muted)` + 透明度
- 焦点环：深/浅主题分别使用低对比描边 + 外扩阴影

**用法示例：**

```html
<input class="yj-field-control" type="search" placeholder="…" />
<select class="yj-field-control">…</select>
```

子页面仅需控制 **布局**（如 `flex:1; max-width`），不要再覆盖 `font-size: 1rem` 等与主页脱节的大小。全局弹窗 **`.modal-title`** 约 **0.82em**（略大于 Tab）、**`.modal-btn`** 约 **0.82em**（与二级页 `.btn` 同级，略大于 Tab）。

## 浮层与弹窗（勿误用表格区半透明变量）

主面板 **表格容器** 使用 `--yj-table-wrap-bg`（深色下多为 **低不透明度**），叠在主窗口渐变上即可；若用于 **Teleport 弹窗、对话框面板**，在 **窗口整体透明** 时会几乎看不见。

约定如下：

| 场景 | 应使用的变量 / 类 |
|------|-------------------|
| 表格、卡片内列表区 | `--yj-table-wrap-bg`、`--yj-table-wrap-border` |
| 右键菜单 | `--yj-ctx-menu-bg`（高不透明度）+ [`MainPanel.vue`](../src/views/MainPanel.vue) `.ctx-menu` |
| 遮罩（scrim） | `--yj-modal-scrim` |
| 弹窗面板底色 | **`--yj-modal-panel-bg`**（近不透明实色）、`--yj-modal-panel-border` |
| 弹窗结构类 | 全局 **`.yj-modal`、`.modal-panel`、`.modal-title`、`.modal-actions`、`.modal-btn`**（定义于 `App.vue`） |

弹窗内输入框请使用 **`yj-field-control`**，不要用 `--yj-tool-bg` 作为大面积输入底（避免「透上叠透」）。

## 页面级约定（已对齐示例）

[`WatchlistAddView.vue`](../src/views/WatchlistAddView.vue)：`yj-field-control` 搜索框、卡片 8px 圆角、代码 DM Sans + tabular-nums。

[`WatchlistManageView.vue`](../src/views/WatchlistManageView.vue)、[`SettingsView.vue`](../src/views/SettingsView.vue)、[`StockDetailView.vue`](../src/views/StockDetailView.vue)：表单控件与主按钮圆角 **8px**，代码/数字列 tabular-nums + DM Sans 为主。

## 与「设置页渐变背景」的关系

二级页面仍可使用 `--yj-settings-bg-1/2` 做轻微渐变，但 **控件与浮层** 必须走 `--yj-field-*`、`--yj-modal-*`、`--yj-ctx-menu-bg` 等语义变量，深浅主题与 **窗口透明度** 下才一致可读。

## 相关文件

| 文件 | 作用 |
|------|------|
| `src/App.vue` | 主题变量、滚动条、`.yj-field-control`、`select.yj-field-control`、全局弹窗样式、`--yj-modal-*` |
| `src/views/MainPanel.vue` | 主面板表格参考；分组新建/重命名弹窗模板（样式在 `App.vue`） |
| `docs/UI_OPTIMIZATION_CHECKLIST.md` | 各页面对照清单（勾选推进） |
