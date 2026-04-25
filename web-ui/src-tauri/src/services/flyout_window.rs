use serde::Serialize;
use tauri::{Emitter, Manager};

use crate::{
    error::{AppError, AppResult},
    models::{MediaSessionDto, SettingsDto},
    services::monitor,
};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToastPayload {
    pub title: String,
    pub subtitle: String,
    pub icon: String,
    pub album_art_data_url: Option<String>,
}

pub fn show_media_flyout<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    settings: &SettingsDto,
) -> AppResult<()> {
    if !settings.media_flyout_enabled {
        return Ok(());
    }

    let width = if settings.compact_layout { 400 } else { 310 };
    let height = if settings.seekbar_enabled { 184 } else { 116 };
    show_window(app, "media-flyout", settings, width, height, settings.duration)
}

pub fn show_next_up_flyout<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    settings: &SettingsDto,
    media: &MediaSessionDto,
) -> AppResult<()> {
    if !settings.next_up_enabled || media.title.trim().is_empty() || media.title == "No media playing" {
        return Ok(());
    }

    let payload = ToastPayload {
        title: media.title.clone(),
        subtitle: media.artist.clone(),
        icon: "next".to_string(),
        album_art_data_url: media.album_art_data_url.clone(),
    };
    show_window(app, "next-up-flyout", settings, 250, 62, settings.next_up_duration)?;
    emit_payload(app, "next-up-flyout", "toast-payload", payload.clone())?;
    emit_payload_delayed(app.clone(), "next-up-flyout", "toast-payload", payload);
    Ok(())
}

pub fn show_lock_keys_flyout<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    settings: &SettingsDto,
    key_name: &str,
    enabled: bool,
) -> AppResult<()> {
    if !settings.lock_keys_enabled {
        return Ok(());
    }

    let payload = ToastPayload {
        title: key_name.to_string(),
        subtitle: if enabled { "켜짐".to_string() } else { "꺼짐".to_string() },
        icon: "lock".to_string(),
        album_art_data_url: None,
    };
    show_window(app, "lock-keys-flyout", settings, 190, 50, settings.lock_keys_duration)?;
    emit_payload(app, "lock-keys-flyout", "toast-payload", payload.clone())?;
    emit_payload_delayed(app.clone(), "lock-keys-flyout", "toast-payload", payload);
    Ok(())
}

fn emit_payload<R: tauri::Runtime, T: Serialize + Clone>(
    app: &tauri::AppHandle<R>,
    label: &str,
    event: &str,
    payload: T,
) -> AppResult<()> {
    if let Some(window) = app.get_webview_window(label) {
        window
            .emit(event, payload)
            .map_err(|error| AppError::with_detail("window.emit", "Flyout 상태를 전달할 수 없습니다.", error))?;
    }
    Ok(())
}

fn emit_payload_delayed<R: tauri::Runtime, T: Serialize + Clone + Send + 'static>(
    app: tauri::AppHandle<R>,
    label: &'static str,
    event: &'static str,
    payload: T,
) {
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(80)).await;
        let _ = emit_payload(&app, label, event, payload);
    });
}

fn show_window<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    label: &str,
    settings: &SettingsDto,
    width: i32,
    height: i32,
    duration_ms: i32,
) -> AppResult<()> {
    let window = app
        .get_webview_window(label)
        .ok_or_else(|| AppError::new("window.missing", format!("{label} 창을 찾을 수 없습니다.")))?;
    let selected = monitor::selected_monitor(settings.flyout_selected_monitor);
    let (x, y) = position_for(&selected, settings.position, width, height);

    window
        .set_size(tauri::Size::Physical(tauri::PhysicalSize { width: width as u32, height: height as u32 }))
        .map_err(|error| AppError::with_detail("window.size", "Flyout 크기를 설정할 수 없습니다.", error))?;
    window
        .set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y }))
        .map_err(|error| AppError::with_detail("window.position", "Flyout 위치를 설정할 수 없습니다.", error))?;
    window
        .show()
        .map_err(|error| AppError::with_detail("window.show", "Flyout을 표시할 수 없습니다.", error))?;
    let _ = window.set_focus();

    if duration_ms > 0 && !settings.media_flyout_always_display {
        let window_for_timer = window.clone();
        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(duration_ms as u64)).await;
            let _ = window_for_timer.hide();
        });
    }

    Ok(())
}

fn position_for(
    monitor: &crate::models::MonitorDto,
    position: i32,
    width: i32,
    height: i32,
) -> (i32, i32) {
    let margin = 56;
    let min_x = monitor.left + 8;
    let max_x = monitor.left + monitor.width - width - 8;
    let min_y = monitor.top + 8;
    let max_y = monitor.top + monitor.height - height - margin;
    let left = monitor.left + 24;
    let center = monitor.left + (monitor.width - width) / 2;
    let right = monitor.left + monitor.width - width - 24;
    let top = monitor.top + 24;
    let bottom = max_y;

    let (x, y) = match position {
        0 => (left, bottom),
        2 => (right, bottom),
        3 => (left, top),
        4 => (center, top),
        5 => (right, top),
        _ => (center, bottom),
    };
    (x.clamp(min_x, max_x.max(min_x)), y.clamp(min_y, max_y.max(min_y)))
}
