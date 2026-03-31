use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, TrayIconBuilder, TrayIconEvent};
use tauri::Manager;
use tauri::WindowEvent;
use tauri_plugin_global_shortcut::{Builder as GsBuilder, GlobalShortcutExt, ShortcutState};

mod quotes;

fn default_theme() -> String {
    "dark".into()
}

fn default_quote_source() -> String {
    "eastmoney".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    #[serde(default = "default_theme")]
    pub theme: String,
    pub font_size_px: u32,
    pub visible_columns: Vec<String>,
    pub panel_mode: String,
    pub auto_hide_edge: bool,
    pub color_scheme: String,
    pub profit_display: String,
    pub opacity: f64,
    pub always_on_top: bool,
    pub skip_taskbar: bool,
    pub boss_shortcut: String,
    pub watchlist: Vec<WatchItem>,
    #[serde(default = "default_watch_groups")]
    pub watch_groups: Vec<WatchGroup>,
    #[serde(default = "default_quote_source")]
    pub quote_source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchItem {
    pub code: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchGroup {
    pub id: String,
    pub name: String,
    pub codes: Vec<String>,
}

fn default_watch_groups() -> Vec<WatchGroup> {
    vec![]
}

fn migrate_watch_groups(settings: &mut AppSettings) {
    if !settings.watch_groups.is_empty() {
        return;
    }
    let id = uuid::Uuid::new_v4().to_string();
    if settings.watchlist.is_empty() {
        settings.watch_groups.push(WatchGroup {
            id,
            name: "自选股".to_string(),
            codes: vec![],
        });
    } else {
        settings.watch_groups.push(WatchGroup {
            id,
            name: "自选股".to_string(),
            codes: settings.watchlist.iter().map(|w| w.code.clone()).collect(),
        });
    }
}

fn migrate_visible_columns(settings: &mut AppSettings) {
    // 旧版两档默认 → 先统一到「换手+量+额」，再由下一段迁移到短线三列
    let old_default_1 = vec!["name", "changePct", "price", "prevClose", "open"];
    let old_default_2 = vec!["name", "price", "changePct", "prevClose", "open"];
    if settings.visible_columns == old_default_1 || settings.visible_columns == old_default_2 {
        settings.visible_columns = vec![
            "name".into(),
            "price".into(),
            "changePct".into(),
            "turnoverRate".into(),
            "volume".into(),
            "turnover".into(),
        ];
    }
    // 仍为「换手 / 成交量 / 成交额」默认组合时，升级为量比 + 板块 + 板块涨幅
    let old_liquidity = vec![
        "name",
        "price",
        "changePct",
        "turnoverRate",
        "volume",
        "turnover",
    ];
    if settings.visible_columns == old_liquidity {
        settings.visible_columns = vec![
            "name".into(),
            "price".into(),
            "changePct".into(),
            "volumeRatio".into(),
            "sectorBlock".into(),
            "sectorPct".into(),
        ];
    }
    // 早期短线默认第三列为「个股振幅」，升级为「板块涨幅」
    let old_short_amp = vec![
        "name",
        "price",
        "changePct",
        "volumeRatio",
        "sectorBlock",
        "amplitude",
    ];
    if settings.visible_columns == old_short_amp {
        settings.visible_columns = vec![
            "name".into(),
            "price".into(),
            "changePct".into(),
            "volumeRatio".into(),
            "sectorBlock".into(),
            "sectorPct".into(),
        ];
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        let watchlist = vec![
            WatchItem {
                code: "sh000001".into(),
                name: "上证指数".into(),
            },
            WatchItem {
                code: "sz399001".into(),
                name: "深证成指".into(),
            },
        ];
        let codes: Vec<String> = watchlist.iter().map(|w| w.code.clone()).collect();
        Self {
            theme: "dark".into(),
            font_size_px: 14,
            visible_columns: vec![
                "name".into(),
                "price".into(),
                "changePct".into(),
                "volumeRatio".into(),
                "sectorBlock".into(),
                "sectorPct".into(),
            ],
            panel_mode: "normal".into(),
            auto_hide_edge: true,
            color_scheme: "redUp".into(),
            profit_display: "both".into(),
            opacity: 0.94,
            always_on_top: true,
            skip_taskbar: false,
            boss_shortcut: "Ctrl+Shift+H".into(),
            watchlist,
            watch_groups: vec![WatchGroup {
                id: uuid::Uuid::new_v4().to_string(),
                name: "自选股".into(),
                codes,
            }],
            quote_source: default_quote_source(),
        }
    }
}

fn config_dir() -> Result<PathBuf, String> {
    dirs::config_dir()
        .ok_or_else(|| "无法解析配置目录".to_string())
        .map(|p| p.join("yuanjingling"))
}

fn settings_path() -> Result<PathBuf, String> {
    Ok(config_dir()?.join("settings.json"))
}

fn ensure_config_dir() -> Result<PathBuf, String> {
    let dir = config_dir()?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

fn load_settings_inner() -> Result<AppSettings, String> {
    let path = settings_path()?;
    if !path.exists() {
        return Ok(AppSettings::default());
    }
    let raw = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let mut s: AppSettings = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    migrate_watch_groups(&mut s);
    migrate_visible_columns(&mut s);
    Ok(s)
}

#[tauri::command]
fn load_settings() -> Result<AppSettings, String> {
    load_settings_inner()
}

#[tauri::command]
fn save_settings(settings: AppSettings) -> Result<AppSettings, String> {
    ensure_config_dir()?;
    let path = settings_path()?;
    let json = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())?;
    Ok(settings)
}

#[tauri::command]
async fn get_quotes(
    codes: Vec<String>,
    quote_source: String,
) -> Result<Vec<quotes::QuoteRow>, String> {
    quotes::get_quotes_impl(codes, &quote_source).await
}

#[tauri::command]
async fn search_securities(query: String) -> Result<Vec<quotes::SuggestItem>, String> {
    quotes::search_securities_eastmoney(&query).await
}

/// 东财股吧个股人气榜（搜索框为空时的推荐）
#[tauri::command]
async fn fetch_hot_stocks() -> Result<Vec<quotes::SuggestItem>, String> {
    quotes::fetch_hot_stocks_eastmoney(30).await
}

#[tauri::command]
async fn get_stock_intraday(
    code: String,
    quote_source: String,
) -> Result<quotes::IntradaySeries, String> {
    quotes::get_stock_intraday_impl(&code, &quote_source).await
}

#[tauri::command]
async fn get_stock_kline(
    code: String,
    period: String,
    quote_source: String,
) -> Result<quotes::KlineSeries, String> {
    quotes::get_stock_kline_impl(&code, &period, &quote_source).await
}

#[tauri::command]
async fn get_stock_order_book(
    code: String,
    quote_source: String,
) -> Result<quotes::OrderBook, String> {
    quotes::get_stock_order_book_impl(&code, &quote_source).await
}

/// 股市异动（东财盘口异动多类合并，与行情 `quote_source` 一致）
#[tauri::command]
async fn get_market_moves(quote_source: String) -> Result<Vec<quotes::MarketMoveItem>, String> {
    quotes::get_market_moves_impl(&quote_source).await
}

/// 主面板底部：行业板块滚动 + 涨跌家数 + 沪深两市成交额/昨成交额
#[tauri::command]
async fn get_market_ribbon(quote_source: String) -> Result<quotes::MarketRibbonSnapshot, String> {
    quotes::get_market_ribbon_impl(&quote_source).await
}

fn apply_window_prefs_internal(
    app: &tauri::AppHandle,
    _opacity: f64,
    always_on_top: bool,
    skip_taskbar: bool,
) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("main") {
        // 整体透明度由前端根壳层（.yj-shell）的 style.opacity 控制（Tauri 2 WebviewWindow 无 set_opacity）
        let _ = win.set_always_on_top(always_on_top);
        let _ = win.set_skip_taskbar(skip_taskbar);
    }
    Ok(())
}

#[tauri::command]
fn apply_window_prefs(
    app: tauri::AppHandle,
    opacity: f64,
    always_on_top: bool,
    skip_taskbar: bool,
) -> Result<(), String> {
    apply_window_prefs_internal(&app, opacity, always_on_top, skip_taskbar)
}

fn show_and_focus_main(app: &tauri::AppHandle) {
    let settings = load_settings_inner().unwrap_or_default();
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.unminimize();
        let _ = w.show();
        let _ = w.set_focus();
    }
    let _ = apply_window_prefs_internal(
        app,
        settings.opacity,
        settings.always_on_top,
        settings.skip_taskbar,
    );
}

/// Windows：置顶 + 无边框透明窗口最小化后任务栏按钮常消失；最小化前暂取消置顶，最小化后按设置重新登记任务栏。
#[tauri::command]
fn minimize_main_window(app: tauri::AppHandle) -> Result<(), String> {
    let settings = load_settings_inner().unwrap_or_default();
    let Some(w) = app.get_webview_window("main") else {
        return Ok(());
    };

    #[cfg(target_os = "windows")]
    {
        if settings.always_on_top {
            w.set_always_on_top(false).map_err(|e| e.to_string())?;
        }
    }

    w.minimize().map_err(|e| e.to_string())?;

    #[cfg(target_os = "windows")]
    {
        if !settings.skip_taskbar {
            w.set_skip_taskbar(false).map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

fn register_boss_shortcut(app: &tauri::AppHandle, shortcut: &str) -> Result<(), String> {
    let gs = app.global_shortcut();
    let _ = gs.unregister_all();
    gs.on_shortcut(shortcut, |app_h, _, event| {
        if event.state != ShortcutState::Pressed {
            return;
        }
        if let Some(w) = app_h.get_webview_window("main") {
            if let Ok(v) = w.is_visible() {
                if v {
                    let _ = w.hide();
                } else {
                    show_and_focus_main(app_h);
                }
            }
        }
    })
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn update_boss_shortcut(app: tauri::AppHandle, shortcut: String) -> Result<(), String> {
    register_boss_shortcut(&app, &shortcut)
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            load_settings,
            save_settings,
            get_quotes,
            search_securities,
            fetch_hot_stocks,
            get_stock_intraday,
            get_stock_kline,
            get_stock_order_book,
            get_market_moves,
            get_market_ribbon,
            apply_window_prefs,
            minimize_main_window,
            update_boss_shortcut
        ])
        .on_window_event(|window, event| {
            #[cfg(target_os = "windows")]
            if window.label() == "main" {
                if let WindowEvent::Focused(true) = event {
                    if window.is_minimized().unwrap_or(false) {
                        return;
                    }
                    let app = window.app_handle().clone();
                    let settings = load_settings_inner().unwrap_or_default();
                    let _ = apply_window_prefs_internal(
                        &app,
                        settings.opacity,
                        settings.always_on_top,
                        settings.skip_taskbar,
                    );
                }
            }
        })
        .setup(|app| {
            let handle = app.handle().clone();

            handle
                .plugin(GsBuilder::new().build())
                .expect("global-shortcut 插件初始化失败");

            let settings = load_settings_inner().unwrap_or_default();
            let _ = apply_window_prefs_internal(
                &handle,
                settings.opacity,
                settings.always_on_top,
                settings.skip_taskbar,
            );
            let _ = register_boss_shortcut(&handle, &settings.boss_shortcut);

            let show_i = MenuItem::with_id(app, "show", "显示主窗口", true, None::<&str>)
                .expect("托盘菜单项");
            let quit_i =
                MenuItem::with_id(app, "quit", "退出", true, None::<&str>).expect("托盘菜单项");
            let menu = Menu::with_items(app, &[&show_i, &quit_i]).expect("托盘菜单");

            let icon = app
                .default_window_icon()
                .expect("缺少应用图标")
                .clone();

            let _tray = TrayIconBuilder::new()
                .icon(icon)
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::DoubleClick { button, .. } = event {
                        if button == MouseButton::Left {
                            show_and_focus_main(tray.app_handle());
                        }
                    }
                })
                .on_menu_event(move |app, event| {
                    match event.id.as_ref() {
                        "quit" => app.exit(0),
                        "show" => show_and_focus_main(app),
                        _ => {}
                    }
                })
                .build(app)
                .expect("托盘图标创建失败");

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
