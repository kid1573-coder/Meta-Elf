use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::Manager;
use tauri_plugin_global_shortcut::{Builder as GsBuilder, GlobalShortcutExt, ShortcutState};

mod quotes;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchItem {
    pub code: String,
    pub name: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            font_size_px: 14,
            visible_columns: vec![
                "name".into(),
                "changePct".into(),
                "price".into(),
                "prevClose".into(),
                "open".into(),
            ],
            panel_mode: "normal".into(),
            auto_hide_edge: true,
            color_scheme: "redUp".into(),
            profit_display: "both".into(),
            opacity: 0.94,
            always_on_top: true,
            skip_taskbar: false,
            boss_shortcut: "Ctrl+Shift+H".into(),
            watchlist: vec![
                WatchItem {
                    code: "sh000001".into(),
                    name: "上证指数".into(),
                },
                WatchItem {
                    code: "sz399001".into(),
                    name: "深证成指".into(),
                },
            ],
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
    serde_json::from_str(&raw).map_err(|e| e.to_string())
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
fn get_quotes() -> Result<Vec<quotes::QuoteRow>, String> {
    Ok(quotes::mock_provider().fetch())
}

fn apply_window_prefs_internal(
    app: &tauri::AppHandle,
    opacity: f64,
    always_on_top: bool,
    skip_taskbar: bool,
) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("main") {
        let o = opacity.clamp(0.35, 1.0);
        let _ = win.set_opacity(o);
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
                    let _ = w.show();
                    let _ = w.set_focus();
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
            apply_window_prefs,
            update_boss_shortcut
        ])
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
                .menu_on_left_click(true)
                .on_menu_event(move |app, event| {
                    match event.id.as_ref() {
                        "quit" => app.exit(0),
                        "show" => {
                            if let Some(w) = app.get_webview_window("main") {
                                let _ = w.show();
                                let _ = w.set_focus();
                            }
                        }
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
