# 界面优化清单（按页勾选）

用于跟踪「与主面板 / `App.vue` 主题变量一致」的改造进度。完成一项可将 `[ ]` 改为 `[x]`。

## 历史优化记录（便于对照体验）

- **主面板表格行右键菜单**：曾过透难辨；已改为使用 **`--yj-ctx-menu-bg`**（高不透明度）+ 实色 fallback、`background-image: none`，与 [`MainPanel.vue`](../src/views/MainPanel.vue) `.ctx-menu` 注释一致。
- **分组 Tab「新建 / 重命名」弹窗**：曾误用 **`--yj-table-wrap-bg`** 作面板底 + 输入区 **`--yj-tool-bg`**，在透明窗口下过透；已改为 **`--yj-modal-scrim` / `--yj-modal-panel-bg`**，弹窗样式提至 [`App.vue`](../src/App.vue) 全局，输入使用 **`yj-field-control`**。
- **弹窗/菜单字号大于 Tab**：因 **`rem` 相对 `html`** 不随 `.yj-shell` 首页字体缩放；已将 **`#yj-root` 内** 弹窗（`.modal-title` / `.modal-btn`）、**`yj-field-control`**、主面板 **右键菜单 / Toast / 工具栏** 等与 Tab 相关的字号改为 **`em`**，与「全部 / 自选股」同级视觉尺度一致。
- **二级页整体字偏大**：顶栏/卡片标题等曾大量用 **`rem`（16px 基准）**，与主面板 **Tab 0.78em / 表头 0.8em / 正文 0.88em** 脱节；已统一为 **`em` 阶梯**（顶栏标题约 **0.86em**、区块 h2 约 **0.8em**、返回约 **0.82em** 等），个股详情大价改为 **`1.55em`** 随「首页字体」缩放。

## 全局（所有含表单的页面）

- [x] **`#yj-root` 内字号用 `em`**：弹窗、菜单、Toast、工具栏、**各二级页顶栏与区块标题**等勿用 `rem` 顶字号；阶梯见 `UI_CONSISTENCY.md` 设计原则第 3、4 条
- [x] 文本类输入统一优先使用 **`class="yj-field-control"`**（`App.vue`：`input` / `textarea`）
- [x] **下拉框**使用 **`select.yj-field-control`**（与输入框同令牌）
- [ ] 避免在组件内写死 `#ffffff` / `#000000` 作为控件前景背景，改用 `--yj-*` 变量
- [x] 数字、代码列：核心页已用 **`tabular-nums`** + **`DM Sans`**（主面板、自选管理、详情、设置老板键/透明度等）；新增页面请延续
- [x] 卡片/表格容器圆角与主面板 **`8px`**：自选管理、设置按钮、详情格子等已从 10px 对齐；新增区块优先 **8px**

## 浮层与弹窗（全局）

- [x] **弹窗遮罩**使用 **`var(--yj-modal-scrim)`**，勿写死过低对比 rgba
- [x] **弹窗面板**使用 **`var(--yj-modal-panel-bg)`**，**禁止**用 `--yj-table-wrap-bg` 作弹窗内容区底色
- [x] **全局类** `.yj-modal` / `.modal-panel` / `.modal-btn` 定义在 `App.vue`，主面板仅保留模板

## 分页面

- [x] **添加自选**（`WatchlistAddView.vue`）：避免大块教程式说明，交互靠占位符与列表即可；仅保留必要上下文（如「当前分组」一行弱提示）
- [x] **自选股管理**（`WatchlistManageView.vue`）：`yj-field-control`、分组 `select`、高级区与卡片 8px、代码列字体
- [x] **设置**（`SettingsView.vue`）：字号下拉与老板键输入 `yj-field-control`、按钮 8px、透明度数字对比略提
- [x] **个股详情**（`StockDetailView.vue`）：数据格与按钮 8px、价格/代码 DM Sans + tabular-nums
- [x] **主面板**（`MainPanel.vue`）：分组弹窗与右键菜单浮层策略已对齐；表格区仍为视觉基准
- [x] **其他**（`BrandElfPreviewView.vue`）：标题/工具条等已改为 `em`，与全局一致；动效预览区仍可按需微调

## 回归检查（改完任一页后建议快速看一遍）

- [ ] 切换到 **浅色主题**，输入框/弹窗/菜单边框与文字是否仍清晰
- [ ] 切换到 **深色主题**，焦点环是否过亮或过暗
- [ ] 窗口 **透明度** 调低时，**分组弹窗**与 **右键菜单** 是否仍清晰可读

---

详细约定说明见：[UI_CONSISTENCY.md](./UI_CONSISTENCY.md)。
