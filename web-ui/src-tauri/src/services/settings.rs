use std::{fs, path::PathBuf};

use serde_json::Value;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    SqlitePool,
};
use tauri::{AppHandle, Manager};

use crate::{
    error::{AppError, AppResult},
    models::SettingsDto,
};

#[derive(Clone)]
pub struct SettingsService {
    pool: SqlitePool,
}

impl SettingsService {
    pub async fn new(app: &AppHandle) -> AppResult<Self> {
        let app_dir = app
            .path()
            .app_data_dir()
            .map_err(|error| AppError::with_detail("settings.path", "설정 저장 경로를 확인할 수 없습니다.", error))?;
        fs::create_dir_all(&app_dir)
            .map_err(|error| AppError::with_detail("settings.path.create", "설정 저장 폴더를 만들 수 없습니다.", error))?;

        let db_path = app_dir.join("fluent_flyout.sqlite");
        let options = SqliteConnectOptions::new()
            .filename(&db_path)
            .create_if_missing(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(4)
            .connect_with(options)
            .await
            .map_err(|error| AppError::with_detail("settings.db.connect", "SQLite 설정 데이터베이스에 연결할 수 없습니다.", error))?;

        sqlx::query("CREATE TABLE IF NOT EXISTS settings (key TEXT PRIMARY KEY NOT NULL, value TEXT NOT NULL, updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP)")
            .execute(&pool)
            .await
            .map_err(|error| AppError::with_detail("settings.db.init", "설정 데이터베이스를 초기화할 수 없습니다.", error))?;

        let service = Self { pool };
        if service.load_raw().await?.is_none() {
            let initial = import_legacy_settings().unwrap_or_default();
            service.save(&initial).await?;
        }
        Ok(service)
    }

    pub async fn load(&self) -> AppResult<SettingsDto> {
        match self.load_raw().await? {
            Some(value) => match serde_json::from_str(&value) {
                Ok(settings) => Ok(settings),
                Err(error) => {
                    eprintln!("설정 JSON 파싱 실패, 기본값으로 복구합니다: {error}");
                    let fallback = import_legacy_settings().unwrap_or_default();
                    if let Err(error) = self.save(&fallback).await {
                        eprintln!("설정 복구 저장 실패: {error}");
                    }
                    Ok(fallback)
                }
            },
            None => {
                let fallback = import_legacy_settings().unwrap_or_default();
                if let Err(error) = self.save(&fallback).await {
                    eprintln!("설정 초기 저장 실패: {error}");
                }
                Ok(fallback)
            }
        }
    }

    pub async fn update(&self, patch: Value) -> AppResult<SettingsDto> {
        let mut value = serde_json::to_value(self.load().await?)
            .map_err(|error| AppError::with_detail("settings.serialize", "설정을 변환할 수 없습니다.", error))?;
        merge_json(&mut value, patch);
        let mut settings: SettingsDto = serde_json::from_value(value)
            .map_err(|error| AppError::with_detail("settings.patch", "설정 값 형식이 올바르지 않습니다.", error))?;
        clamp_settings(&mut settings);
        self.save(&settings).await?;
        Ok(settings)
    }

    pub async fn save(&self, settings: &SettingsDto) -> AppResult<()> {
        let json = serde_json::to_string(settings)
            .map_err(|error| AppError::with_detail("settings.serialize", "설정을 저장 형식으로 변환할 수 없습니다.", error))?;
        sqlx::query("INSERT INTO settings(key, value, updated_at) VALUES('settings', ?1, CURRENT_TIMESTAMP) ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = CURRENT_TIMESTAMP")
            .bind(json)
            .execute(&self.pool)
            .await
            .map_err(|error| AppError::with_detail("settings.db.save", "설정을 저장할 수 없습니다.", error))?;
        save_legacy_settings(settings)?;
        Ok(())
    }

    async fn load_raw(&self) -> AppResult<Option<String>> {
        let row: Option<(String,)> = sqlx::query_as("SELECT value FROM settings WHERE key = 'settings'")
            .fetch_optional(&self.pool)
            .await
            .map_err(|error| AppError::with_detail("settings.db.load", "설정을 불러올 수 없습니다.", error))?;
        Ok(row.map(|row| row.0))
    }
}

fn merge_json(base: &mut Value, patch: Value) {
    if let (Some(base), Some(patch)) = (base.as_object_mut(), patch.as_object()) {
        for (key, value) in patch {
            base.insert(key.clone(), value.clone());
        }
    }
}

fn clamp_settings(settings: &mut SettingsDto) {
    settings.duration = settings.duration.clamp(0, 10_000);
    settings.next_up_duration = settings.next_up_duration.clamp(0, 10_000);
    settings.lock_keys_duration = settings.lock_keys_duration.clamp(0, 10_000);
    settings.acrylic_blur_opacity = settings.acrylic_blur_opacity.clamp(0, 255);
    settings.taskbar_visualizer_bar_count = settings.taskbar_visualizer_bar_count.clamp(1, 20);
    settings.taskbar_visualizer_audio_sensitivity = settings.taskbar_visualizer_audio_sensitivity.clamp(1, 3);
    settings.taskbar_visualizer_audio_peak_level = settings.taskbar_visualizer_audio_peak_level.clamp(1, 3);
}

fn import_legacy_settings() -> Option<SettingsDto> {
    let path = legacy_settings_path()?;
    let xml = fs::read_to_string(path).ok()?;
    let mut settings = SettingsDto::default();
    apply_bool(&xml, "MediaFlyoutEnabled", &mut settings.media_flyout_enabled);
    apply_i32(&xml, "MediaFlyoutBackgroundBlur", &mut settings.media_flyout_background_blur);
    apply_bool(&xml, "MediaFlyoutAcrylicWindowEnabled", &mut settings.media_flyout_acrylic_window_enabled);
    apply_bool(&xml, "CompactLayout", &mut settings.compact_layout);
    apply_i32(&xml, "Position", &mut settings.position);
    apply_i32(&xml, "Duration", &mut settings.duration);
    apply_bool(&xml, "MediaFlyoutAlwaysDisplay", &mut settings.media_flyout_always_display);
    apply_bool(&xml, "CenterTitleArtist", &mut settings.center_title_artist);
    apply_bool(&xml, "PlayerInfoEnabled", &mut settings.player_info_enabled);
    apply_bool(&xml, "RepeatEnabled", &mut settings.repeat_enabled);
    apply_bool(&xml, "ShuffleEnabled", &mut settings.shuffle_enabled);
    apply_bool(&xml, "SeekbarEnabled", &mut settings.seekbar_enabled);
    apply_bool(&xml, "PauseOtherSessionsEnabled", &mut settings.pause_other_sessions_enabled);
    apply_bool(&xml, "MediaFlyoutVolumeKeysExcluded", &mut settings.media_flyout_volume_keys_excluded);
    apply_bool(&xml, "ExclusiveTidalMode", &mut settings.exclusive_tidal_mode);
    apply_bool(&xml, "NextUpEnabled", &mut settings.next_up_enabled);
    apply_i32(&xml, "NextUpDuration", &mut settings.next_up_duration);
    apply_bool(&xml, "LockKeysEnabled", &mut settings.lock_keys_enabled);
    apply_i32(&xml, "LockKeysDuration", &mut settings.lock_keys_duration);
    apply_bool(&xml, "LockKeysBoldUI", &mut settings.lock_keys_bold_ui);
    apply_i32(&xml, "LockKeysMonitorPreference", &mut settings.lock_keys_monitor_preference);
    apply_bool(&xml, "LockKeysInsertEnabled", &mut settings.lock_keys_insert_enabled);
    apply_bool(&xml, "TaskbarWidgetEnabled", &mut settings.taskbar_widget_enabled);
    apply_i32(&xml, "TaskbarWidgetPosition", &mut settings.taskbar_widget_position);
    apply_i32(&xml, "TaskbarWidgetSelectedMonitor", &mut settings.taskbar_widget_selected_monitor);
    apply_bool(&xml, "TaskbarWidgetPadding", &mut settings.taskbar_widget_padding);
    apply_i32(&xml, "TaskbarWidgetManualPadding", &mut settings.taskbar_widget_manual_padding);
    apply_bool(&xml, "TaskbarWidgetClickable", &mut settings.taskbar_widget_clickable);
    apply_bool(&xml, "TaskbarWidgetCloseableFlyout", &mut settings.taskbar_widget_closeable_flyout);
    apply_bool(&xml, "TaskbarWidgetBackgroundBlur", &mut settings.taskbar_widget_background_blur);
    apply_bool(&xml, "TaskbarWidgetHideCompletely", &mut settings.taskbar_widget_hide_completely);
    apply_bool(&xml, "TaskbarWidgetControlsEnabled", &mut settings.taskbar_widget_controls_enabled);
    apply_i32(&xml, "TaskbarWidgetControlsPosition", &mut settings.taskbar_widget_controls_position);
    apply_bool(&xml, "TaskbarVisualizerEnabled", &mut settings.taskbar_visualizer_enabled);
    apply_i32(&xml, "TaskbarVisualizerPosition", &mut settings.taskbar_visualizer_position);
    apply_i32(&xml, "TaskbarVisualizerBarCount", &mut settings.taskbar_visualizer_bar_count);
    apply_bool(&xml, "TaskbarVisualizerCenteredBars", &mut settings.taskbar_visualizer_centered_bars);
    apply_bool(&xml, "TaskbarVisualizerBaseline", &mut settings.taskbar_visualizer_baseline);
    apply_i32(&xml, "TaskbarVisualizerAudioSensitivity", &mut settings.taskbar_visualizer_audio_sensitivity);
    apply_i32(&xml, "TaskbarVisualizerAudioPeakLevel", &mut settings.taskbar_visualizer_audio_peak_level);
    apply_i32(&xml, "FlyoutSelectedMonitor", &mut settings.flyout_selected_monitor);
    apply_i32(&xml, "AcrylicBlurOpacity", &mut settings.acrylic_blur_opacity);
    apply_i32(&xml, "AppTheme", &mut settings.app_theme);
    apply_bool(&xml, "Startup", &mut settings.startup);
    apply_bool(&xml, "DisableIfFullscreen", &mut settings.disable_if_fullscreen);
    apply_i32(&xml, "nIconLeftClick", &mut settings.nicon_left_click);
    apply_bool(&xml, "nIconSymbol", &mut settings.nicon_symbol);
    apply_bool(&xml, "NIconHide", &mut settings.nicon_hide);
    clamp_settings(&mut settings);
    Some(settings)
}

fn legacy_settings_path() -> Option<PathBuf> {
    dirs::config_dir().map(|path| path.join("FluentFlyout").join("settings.xml"))
}

fn save_legacy_settings(settings: &SettingsDto) -> AppResult<()> {
    let path = legacy_settings_path().ok_or_else(|| {
        AppError::new(
            "settings.legacy.path",
            "원본 WPF 설정 파일 경로를 확인할 수 없습니다.",
        )
    })?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            AppError::with_detail(
                "settings.legacy.create_dir",
                "원본 WPF 설정 폴더를 만들 수 없습니다.",
                error,
            )
        })?;
    }

    let language = match settings.app_language_index {
        1 => "ko",
        2 => "en-US",
        _ => "system",
    };

    let xml = format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<UserSettings xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:xsd="http://www.w3.org/2001/XMLSchema">
  <CompactLayout>{}</CompactLayout>
  <FlyoutSelectedMonitor>{}</FlyoutSelectedMonitor>
  <Position>{}</Position>
  <FlyoutAnimationSpeed>{}</FlyoutAnimationSpeed>
  <PlayerInfoEnabled>{}</PlayerInfoEnabled>
  <RepeatEnabled>{}</RepeatEnabled>
  <ShuffleEnabled>{}</ShuffleEnabled>
  <Startup>{}</Startup>
  <MediaFlyoutAlwaysDisplay>{}</MediaFlyoutAlwaysDisplay>
  <Duration>{}</Duration>
  <NextUpEnabled>{}</NextUpEnabled>
  <NextUpDuration>{}</NextUpDuration>
  <nIconLeftClick>{}</nIconLeftClick>
  <CenterTitleArtist>{}</CenterTitleArtist>
  <FlyoutAnimationEasingStyle>{}</FlyoutAnimationEasingStyle>
  <LockKeysEnabled>{}</LockKeysEnabled>
  <LockKeysDuration>{}</LockKeysDuration>
  <AppTheme>{}</AppTheme>
  <MediaFlyoutEnabled>{}</MediaFlyoutEnabled>
  <MediaFlyoutVolumeKeysExcluded>{}</MediaFlyoutVolumeKeysExcluded>
  <nIconSymbol>{}</nIconSymbol>
  <NIconHide>{}</NIconHide>
  <DisableIfFullscreen>{}</DisableIfFullscreen>
  <LockKeysBoldUI>{}</LockKeysBoldUI>
  <LockKeysMonitorPreference>{}</LockKeysMonitorPreference>
  <LastKnownVersion>debug</LastKnownVersion>
  <ExclusiveTidalMode>{}</ExclusiveTidalMode>
  <SeekbarEnabled>{}</SeekbarEnabled>
  <PauseOtherSessionsEnabled>{}</PauseOtherSessionsEnabled>
  <LockKeysInsertEnabled>{}</LockKeysInsertEnabled>
  <MediaFlyoutBackgroundBlur>{}</MediaFlyoutBackgroundBlur>
  <MediaFlyoutAcrylicWindowEnabled>{}</MediaFlyoutAcrylicWindowEnabled>
  <NextUpAcrylicWindowEnabled>{}</NextUpAcrylicWindowEnabled>
  <LockKeysAcrylicWindowEnabled>{}</LockKeysAcrylicWindowEnabled>
  <AppLanguage>{}</AppLanguage>
  <TaskbarWidgetEnabled>{}</TaskbarWidgetEnabled>
  <TaskbarWidgetSelectedMonitor>{}</TaskbarWidgetSelectedMonitor>
  <TaskbarWidgetPosition>{}</TaskbarWidgetPosition>
  <TaskbarWidgetPadding>{}</TaskbarWidgetPadding>
  <TaskbarWidgetManualPadding>{}</TaskbarWidgetManualPadding>
  <TaskbarWidgetClickable>{}</TaskbarWidgetClickable>
  <TaskbarWidgetCloseableFlyout>{}</TaskbarWidgetCloseableFlyout>
  <TaskbarWidgetBackgroundBlur>{}</TaskbarWidgetBackgroundBlur>
  <TaskbarWidgetHideCompletely>{}</TaskbarWidgetHideCompletely>
  <TaskbarWidgetControlsEnabled>{}</TaskbarWidgetControlsEnabled>
  <TaskbarWidgetControlsPosition>{}</TaskbarWidgetControlsPosition>
  <TaskbarWidgetAnimated>{}</TaskbarWidgetAnimated>
  <TaskbarVisualizerEnabled>{}</TaskbarVisualizerEnabled>
  <TaskbarVisualizerPosition>{}</TaskbarVisualizerPosition>
  <TaskbarVisualizerClickable>{}</TaskbarVisualizerClickable>
  <TaskbarVisualizerBarCount>{}</TaskbarVisualizerBarCount>
  <TaskbarVisualizerCenteredBars>{}</TaskbarVisualizerCenteredBars>
  <TaskbarVisualizerBaseline>{}</TaskbarVisualizerBaseline>
  <TaskbarVisualizerAudioSensitivity>{}</TaskbarVisualizerAudioSensitivity>
  <TaskbarVisualizerAudioPeakLevel>{}</TaskbarVisualizerAudioPeakLevel>
  <AcrylicBlurOpacity>{}</AcrylicBlurOpacity>
  <UseAlbumArtAsAccentColor>{}</UseAlbumArtAsAccentColor>
  <IsStoreVersion>false</IsStoreVersion>
  <LastUpdateNotificationUnixSeconds>0</LastUpdateNotificationUnixSeconds>
  <ShowUpdateNotifications>{}</ShowUpdateNotifications>
  <LegacyTaskbarWidthEnabled>{}</LegacyTaskbarWidthEnabled>
</UserSettings>
"#,
        settings.compact_layout,
        settings.flyout_selected_monitor,
        settings.position,
        settings.flyout_animation_speed,
        settings.player_info_enabled,
        settings.repeat_enabled,
        settings.shuffle_enabled,
        settings.startup,
        settings.media_flyout_always_display,
        settings.duration,
        settings.next_up_enabled,
        settings.next_up_duration,
        settings.nicon_left_click,
        settings.center_title_artist,
        settings.flyout_animation_easing_style,
        settings.lock_keys_enabled,
        settings.lock_keys_duration,
        settings.app_theme,
        settings.media_flyout_enabled,
        settings.media_flyout_volume_keys_excluded,
        settings.nicon_symbol,
        settings.nicon_hide,
        settings.disable_if_fullscreen,
        settings.lock_keys_bold_ui,
        settings.lock_keys_monitor_preference,
        settings.exclusive_tidal_mode,
        settings.seekbar_enabled,
        settings.pause_other_sessions_enabled,
        settings.lock_keys_insert_enabled,
        settings.media_flyout_background_blur,
        settings.media_flyout_acrylic_window_enabled,
        settings.next_up_acrylic_window_enabled,
        settings.lock_keys_acrylic_window_enabled,
        language,
        settings.taskbar_widget_enabled,
        settings.taskbar_widget_selected_monitor,
        settings.taskbar_widget_position,
        settings.taskbar_widget_padding,
        settings.taskbar_widget_manual_padding,
        settings.taskbar_widget_clickable,
        settings.taskbar_widget_closeable_flyout,
        settings.taskbar_widget_background_blur,
        settings.taskbar_widget_hide_completely,
        settings.taskbar_widget_controls_enabled,
        settings.taskbar_widget_controls_position,
        settings.taskbar_widget_animated,
        settings.taskbar_visualizer_enabled,
        settings.taskbar_visualizer_position,
        settings.taskbar_visualizer_clickable,
        settings.taskbar_visualizer_bar_count,
        settings.taskbar_visualizer_centered_bars,
        settings.taskbar_visualizer_baseline,
        settings.taskbar_visualizer_audio_sensitivity,
        settings.taskbar_visualizer_audio_peak_level,
        settings.acrylic_blur_opacity,
        settings.use_album_art_as_accent_color,
        settings.show_update_notifications,
        settings.legacy_taskbar_width_enabled,
    );

    fs::write(&path, xml).map_err(|error| {
        AppError::with_detail(
            "settings.legacy.save",
            "원본 WPF 설정 파일을 저장할 수 없습니다.",
            error,
        )
    })?;
    Ok(())
}

fn tag_value(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = xml.find(&open)? + open.len();
    let end = xml[start..].find(&close)? + start;
    Some(xml[start..end].trim().to_string())
}

fn apply_bool(xml: &str, tag: &str, target: &mut bool) {
    if let Some(value) = tag_value(xml, tag) {
        if let Ok(parsed) = value.parse::<bool>() {
            *target = parsed;
        }
    }
}

fn apply_i32(xml: &str, tag: &str, target: &mut i32) {
    if let Some(value) = tag_value(xml, tag) {
        if let Ok(parsed) = value.parse::<i32>() {
            *target = parsed;
        }
    }
}
