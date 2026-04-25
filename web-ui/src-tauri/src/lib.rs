mod commands;
mod error;
mod models;
mod services;

use commands::*;
use services::{display_host, settings::SettingsService, taskbar_widget, tray_icon};
use tauri::{menu::MenuBuilder, tray::TrayIconBuilder, Manager};

#[derive(Clone)]
pub struct AppState {
    settings: SettingsService,
}

pub fn run() {
    configure_dpi_awareness();

    tauri::Builder::default()
        .setup(|app| {
            let settings = tauri::async_runtime::block_on(SettingsService::new(&app.handle()))?;
            app.manage(AppState { settings: settings.clone() });
            setup_tray(app)?;
            if let Ok(current) = tauri::async_runtime::block_on(settings.load()) {
                let _ = tray_icon::apply_tray_icon(&app.handle(), &current);
            }
            cleanup_taskbar_widget(&app.handle());
            if let Err(error) = display_host::start() {
                eprintln!("표시 전용 WPF 호스트 실행 실패: {error}");
            }
            Ok(())
        })
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { .. } if window.label() == "main" => {
                cleanup_taskbar_widget(window.app_handle());
                display_host::stop_if_owned();
                window.app_handle().exit(0);
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            get_settings,
            update_settings,
            get_media_session,
            media_control,
            show_media_flyout,
            show_next_up_flyout,
            show_lock_keys_flyout,
            set_taskbar_widget_enabled,
            reposition_taskbar_widget,
            list_monitors,
            app_action
        ])
        .run(tauri::generate_context!())
        .expect("Tauri 실행 중 오류가 발생했습니다.");
}

fn setup_tray<R: tauri::Runtime>(app: &tauri::App<R>) -> tauri::Result<()> {
    let menu = MenuBuilder::new(app)
        .text("open_settings", "설정 열기")
        .text("show_flyout", "미디어 Flyout 표시")
        .separator()
        .text("open_logs", "로그 보기")
        .text("open_repository", "GitHub 저장소")
        .separator()
        .text("quit", "종료")
        .build()?;

    let icon = app.default_window_icon().cloned();
    let mut builder = TrayIconBuilder::with_id("main")
        .menu(&menu)
        .tooltip("FluentFlyout")
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| match event.id().as_ref() {
            "open_settings" => show_main_window(app),
            "show_flyout" => {
                if let Err(error) = display_host::show_media_flyout() {
                    eprintln!("미디어 Flyout 표시 실패: {error}");
                }
            }
            "open_logs" => open_logs_folder(app),
            "open_repository" => open_url("https://github.com/minseokk7/FluentFlyout"),
            "quit" => {
                cleanup_taskbar_widget(app);
                display_host::stop_if_owned();
                app.exit(0);
            }
            _ => {}
        });

    if let Some(icon) = icon {
        builder = builder.icon(icon);
    }

    builder.build(app)?;
    Ok(())
}

fn cleanup_taskbar_widget<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    taskbar_widget::destroy_widget_on_main_thread(app);
    taskbar_widget::destroy_orphan_widgets();
}

fn show_main_window<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn open_logs_folder<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    if let Ok(path) = app.path().app_log_dir() {
        let _ = std::fs::create_dir_all(&path);
        open_path(&path.to_string_lossy());
    }
}

fn open_url(url: &str) {
    let _ = std::process::Command::new("cmd").args(["/C", "start", "", url]).spawn();
}

fn open_path(path: &str) {
    let _ = std::process::Command::new("explorer").arg(path).spawn();
}

#[cfg(windows)]
fn configure_dpi_awareness() {
    use windows::Win32::UI::HiDpi::{
        SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
    };
    unsafe {
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    }
}

#[cfg(not(windows))]
fn configure_dpi_awareness() {}
