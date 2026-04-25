use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

use tauri::image::Image;

use crate::{
    error::{AppError, AppResult},
    models::SettingsDto,
};

pub fn apply_tray_icon<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    settings: &SettingsDto,
) -> AppResult<()> {
    let Some(tray) = app.tray_by_id("main") else {
        return Ok(());
    };

    if settings.nicon_hide {
        tray.set_icon(None).map_err(|error| {
            AppError::with_detail("tray.hide", "트레이 아이콘을 숨길 수 없습니다.", error)
        })?;
        return Ok(());
    }

    let icon_path = icon_path(settings)?;
    let icon = Image::from_path(&icon_path).map_err(|error| {
        AppError::with_detail(
            "tray.icon.load",
            "트레이 아이콘 이미지를 불러올 수 없습니다.",
            format!("{}: {}", icon_path.display(), error),
        )
    })?;

    tray.set_icon(Some(icon)).map_err(|error| {
        AppError::with_detail("tray.icon.apply", "트레이 아이콘을 적용할 수 없습니다.", error)
    })
}

fn icon_path(settings: &SettingsDto) -> AppResult<PathBuf> {
    let root = find_repo_root().ok_or_else(|| {
        AppError::new(
            "tray.icon.repo_not_found",
            "원본 트레이 아이콘 리소스 경로를 찾을 수 없습니다.",
        )
    })?;

    let resources = root.join("FluentFlyoutWPF").join("Resources");
    if settings.nicon_symbol {
        let file_name = if prefers_light_tray_icon(settings) {
            "FluentFlyoutBlack.png"
        } else {
            "FluentFlyoutWhite.png"
        };
        Ok(resources.join("TrayIcons").join(file_name))
    } else {
        Ok(resources.join("FluentFlyout2.ico"))
    }
}

fn prefers_light_tray_icon(settings: &SettingsDto) -> bool {
    match settings.app_theme {
        1 => true,
        2 => false,
        _ => windows_apps_use_light_theme().unwrap_or(false),
    }
}

fn windows_apps_use_light_theme() -> Option<bool> {
    let output = Command::new("reg")
        .args([
            "query",
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize",
            "/v",
            "AppsUseLightTheme",
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8_lossy(&output.stdout);
    if text.contains("0x1") {
        Some(true)
    } else if text.contains("0x0") {
        Some(false)
    } else {
        None
    }
}

fn find_repo_root() -> Option<PathBuf> {
    let mut starts = Vec::new();

    if let Ok(current_dir) = env::current_dir() {
        starts.push(current_dir);
    }

    if let Ok(current_exe) = env::current_exe() {
        if let Some(parent) = current_exe.parent() {
            starts.push(parent.to_path_buf());
        }
    }

    starts.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")));

    starts
        .into_iter()
        .flat_map(|start| start.ancestors().map(Path::to_path_buf).collect::<Vec<_>>())
        .find(|candidate| {
            candidate
                .join("FluentFlyoutWPF")
                .join("Resources")
                .join("FluentFlyout2.ico")
                .exists()
        })
}
