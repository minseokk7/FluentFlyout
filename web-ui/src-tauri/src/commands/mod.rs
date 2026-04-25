use serde_json::Value;
use tauri::{Manager, State};

use crate::{
    AppState,
    error::{AppError, AppResult},
    models::{
        FlyoutShowOptions, MediaSessionDto, MonitorDto, SettingsDto, TaskbarWidgetPlacementDto,
    },
    services::{display_host, media_session, monitor, startup, taskbar_widget, tray_icon},
};

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> AppResult<SettingsDto> {
    state.settings.load().await
}

#[tauri::command]
pub async fn update_settings(
    patch: Value,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> AppResult<SettingsDto> {
    let startup_changed = patch.get("startup").is_some();
    let updated = state.settings.update(patch).await?;

    if startup_changed {
        startup::apply_startup_setting(&app, &updated)?;
    }

    tray_icon::apply_tray_icon(&app, &updated)?;
    destroy_rust_taskbar_widget(&app);
    display_host::reload_settings()?;

    Ok(updated)
}

#[tauri::command]
pub async fn get_media_session() -> AppResult<MediaSessionDto> {
    media_session::get_media_session().await
}

#[tauri::command]
pub async fn media_control(action: String) -> AppResult<()> {
    media_session::control(action).await
}

#[tauri::command]
pub async fn show_media_flyout(
    _options: FlyoutShowOptions,
    _app: tauri::AppHandle,
    _state: State<'_, AppState>,
) -> AppResult<()> {
    display_host::show_media_flyout()
}

#[tauri::command]
pub async fn show_next_up_flyout(
    _app: tauri::AppHandle,
    _state: State<'_, AppState>,
) -> AppResult<()> {
    display_host::show_next_up_flyout()
}

#[tauri::command]
pub async fn show_lock_keys_flyout(
    _key_name: String,
    _enabled: bool,
    _app: tauri::AppHandle,
    _state: State<'_, AppState>,
) -> AppResult<()> {
    display_host::show_lock_keys_flyout()
}

#[tauri::command]
pub async fn set_taskbar_widget_enabled(
    enabled: bool,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> AppResult<()> {
    let _ = state
        .settings
        .update(serde_json::json!({ "taskbarWidgetEnabled": enabled }))
        .await?;
    let updated = state.settings.load().await?;
    tray_icon::apply_tray_icon(&app, &updated)?;
    destroy_rust_taskbar_widget(&app);
    display_host::reload_settings()?;
    Ok(())
}

#[tauri::command]
pub async fn reposition_taskbar_widget(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> AppResult<TaskbarWidgetPlacementDto> {
    let _ = state.settings.load().await?;
    destroy_rust_taskbar_widget(&app);
    display_host::start()?;
    Ok(TaskbarWidgetPlacementDto {
        monitor_id: 0,
        taskbar_hwnd: "wpf-display-host".to_string(),
        x: 0,
        y: 0,
        width: 0,
        height: 0,
        dpi_scale: 1.0,
        logical_width: 0,
        logical_height: 0,
        container_x: 0,
        container_y: 0,
        container_width: 0,
        container_height: 0,
        widget_x: 0,
        widget_y: 0,
        source: "wpf-display-host".to_string(),
    })
}

#[tauri::command]
pub async fn list_monitors() -> AppResult<Vec<MonitorDto>> {
    Ok(monitor::list_monitors())
}

#[tauri::command]
pub async fn app_action(
    action: String,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> AppResult<()> {
    match action.as_str() {
        "open_repository" => open_url("https://github.com/minseokk7/FluentFlyout"),
        "open_weblate" => open_url("https://hosted.weblate.org/engage/fluentflyout/"),
        "open_sponsor" => open_url("https://github.com/sponsors/minseokk7"),
        "open_coffee" => open_url("https://www.buymeacoffee.com/minseok"),
        "open_audio_settings" => open_url("ms-settings:apps-volume"),
        "open_advanced_settings" => open_url("ms-settings:startupapps"),
        "open_logs" => open_logs_folder(&app)?,
        "backup_settings" => backup_settings(&app, &state).await?,
        "show_media_flyout" => display_host::show_media_flyout()?,
        "show_next_up_flyout" => display_host::show_next_up_flyout()?,
        "show_lock_keys_flyout" => display_host::show_lock_keys_flyout()?,
        "reposition_taskbar_widget" => {
            destroy_rust_taskbar_widget(&app);
            display_host::reload_settings()?;
        }
        _ => return Err(AppError::new("action.unknown", "지원하지 않는 동작입니다.")),
    }
    Ok(())
}

fn destroy_rust_taskbar_widget(app: &tauri::AppHandle) {
    taskbar_widget::destroy_widget_on_main_thread(app);
    taskbar_widget::destroy_orphan_widgets();
}

fn open_url(url: &str) {
    let _ = std::process::Command::new("cmd")
        .args(["/C", "start", "", url])
        .spawn();
}

fn open_logs_folder(app: &tauri::AppHandle) -> AppResult<()> {
    let path = app
        .path()
        .app_log_dir()
        .map_err(|error| {
            AppError::with_detail("logs.path", "로그 폴더 경로를 확인할 수 없습니다.", error)
        })?;
    std::fs::create_dir_all(&path).map_err(|error| {
        AppError::with_detail("logs.create", "로그 폴더를 만들 수 없습니다.", error)
    })?;
    let _ = std::process::Command::new("explorer").arg(path).spawn();
    Ok(())
}

async fn backup_settings(app: &tauri::AppHandle, state: &State<'_, AppState>) -> AppResult<()> {
    let settings = state.settings.load().await?;
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| {
            AppError::with_detail("backup.path", "백업 저장 경로를 확인할 수 없습니다.", error)
        })?;
    std::fs::create_dir_all(&app_dir).map_err(|error| {
        AppError::with_detail("backup.create_dir", "백업 폴더를 만들 수 없습니다.", error)
    })?;
    let backup_path = app_dir.join("settings-backup.json");
    let json = serde_json::to_string_pretty(&settings).map_err(|error| {
        AppError::with_detail(
            "backup.serialize",
            "설정을 백업 형식으로 변환할 수 없습니다.",
            error,
        )
    })?;
    std::fs::write(&backup_path, json).map_err(|error| {
        AppError::with_detail("backup.write", "설정 백업 파일을 저장할 수 없습니다.", error)
    })?;
    let _ = std::process::Command::new("explorer").arg(app_dir).spawn();
    Ok(())
}
