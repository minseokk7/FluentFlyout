use serde::Deserialize;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename = "UserSettings", rename_all = "PascalCase")]
pub struct FluentFlyoutSettings {
    pub compact_layout: bool,
    pub flyout_selected_monitor: i32,
    pub position: i32,
    pub flyout_animation_speed: i32,
    pub player_info_enabled: bool,
    pub repeat_enabled: bool,
    pub shuffle_enabled: bool,
    pub media_flyout_always_display: bool,
    pub duration: i32,
    pub center_title_artist: bool,
    pub flyout_animation_easing_style: i32,
    pub media_flyout_enabled: bool,
    pub media_flyout_volume_keys_excluded: bool,
    pub disable_if_fullscreen: bool,
    #[serde(rename = "nIconLeftClick")]
    pub nicon_left_click: i32,
    #[serde(rename = "NIconHide")]
    pub nicon_hide: bool,
    pub exclusive_tidal_mode: bool,
    pub seekbar_enabled: bool,
    pub lock_keys_enabled: bool,
    pub lock_keys_duration: i32,
    pub lock_keys_bold_ui: bool,
    pub lock_keys_monitor_preference: i32,
    pub lock_keys_insert_enabled: bool,
    pub media_flyout_background_blur: i32,
    pub media_flyout_acrylic_window_enabled: bool,
    pub lock_keys_acrylic_window_enabled: bool,
    pub startup: bool,
    pub app_theme: i32,
    pub next_up_enabled: bool,
    pub next_up_duration: i32,
    pub next_up_acrylic_window_enabled: bool,
    pub taskbar_widget_enabled: bool,
    pub taskbar_widget_selected_monitor: i32,
    pub taskbar_widget_position: i32,
    pub taskbar_widget_padding: bool,
    pub taskbar_widget_manual_padding: i32,
    pub taskbar_widget_clickable: bool,
    pub taskbar_widget_closeable_flyout: bool,
    pub taskbar_widget_background_blur: bool,
    pub taskbar_widget_hide_completely: bool,
    pub taskbar_widget_controls_enabled: bool,
    pub taskbar_widget_controls_position: i32,
    pub taskbar_widget_animated: bool,
    pub app_language: String,
}

impl Default for FluentFlyoutSettings {
    fn default() -> Self {
        Self {
            compact_layout: false,
            flyout_selected_monitor: 0,
            position: 0,
            flyout_animation_speed: 2,
            player_info_enabled: true,
            repeat_enabled: false,
            shuffle_enabled: false,
            media_flyout_always_display: false,
            duration: 3000,
            center_title_artist: false,
            flyout_animation_easing_style: 2,
            media_flyout_enabled: true,
            media_flyout_volume_keys_excluded: false,
            disable_if_fullscreen: true,
            nicon_left_click: 0,
            nicon_hide: false,
            exclusive_tidal_mode: true,
            seekbar_enabled: false,
            lock_keys_enabled: true,
            lock_keys_duration: 2000,
            lock_keys_bold_ui: false,
            lock_keys_monitor_preference: 0,
            lock_keys_insert_enabled: true,
            media_flyout_background_blur: 0,
            media_flyout_acrylic_window_enabled: true,
            lock_keys_acrylic_window_enabled: true,
            startup: true,
            app_theme: 0,
            next_up_enabled: true,
            next_up_duration: 2000,
            next_up_acrylic_window_enabled: true,
            taskbar_widget_enabled: true,
            taskbar_widget_selected_monitor: 0,
            taskbar_widget_position: 0,
            taskbar_widget_padding: true,
            taskbar_widget_manual_padding: 0,
            taskbar_widget_clickable: true,
            taskbar_widget_closeable_flyout: true,
            taskbar_widget_background_blur: false,
            taskbar_widget_hide_completely: false,
            taskbar_widget_controls_enabled: true,
            taskbar_widget_controls_position: 1,
            taskbar_widget_animated: true,
            app_language: "system".to_owned(),
        }
    }
}

pub struct MediaFlyoutPreviewState {
    pub title: &'static str,
    pub artist: &'static str,
    pub player: &'static str,
    pub current_duration: &'static str,
    pub max_duration: &'static str,
}

pub struct AppPreviewState {
    pub normal: MediaFlyoutPreviewState,
    pub compact: MediaFlyoutPreviewState,
    pub flyout_position: i32,
    pub flyout_selected_monitor: i32,
    pub flyout_animation_speed: i32,
    pub flyout_animation_easing_style: i32,
    pub flyout_duration_ms: u64,
    pub media_flyout_enabled: bool,
    pub media_flyout_volume_keys_excluded: bool,
    pub disable_if_fullscreen: bool,
    pub nicon_left_click: i32,
    pub nicon_hide: bool,
    pub startup: bool,
    pub app_theme: i32,
    pub next_up_enabled: bool,
    pub next_up_duration_ms: u64,
    pub next_up_acrylic_window_enabled: bool,
    pub taskbar_widget_enabled: bool,
    pub taskbar_widget_selected_monitor: i32,
    pub taskbar_widget_position: i32,
    pub taskbar_widget_padding: bool,
    pub taskbar_widget_manual_padding: i32,
    pub taskbar_widget_clickable: bool,
    pub taskbar_widget_closeable_flyout: bool,
    pub taskbar_widget_background_blur: bool,
    pub taskbar_widget_hide_completely: bool,
    pub taskbar_widget_controls_enabled: bool,
    pub taskbar_widget_controls_position: i32,
    pub taskbar_widget_animated: bool,
    pub normal_compact_layout: bool,
    pub normal_center_title_artist: bool,
    pub normal_play_pause_label: String,
    pub normal_repeat_label: String,
    pub normal_repeat_active: bool,
    pub normal_repeat_available: bool,
    pub normal_shuffle_active: bool,
    pub normal_shuffle_available: bool,
    pub normal_seekbar_value: f32,
    pub normal_seekbar_maximum: f32,
    pub normal_repeat_enabled: bool,
    pub normal_shuffle_enabled: bool,
    pub normal_player_info_enabled: bool,
    pub normal_seekbar_enabled: bool,
    pub normal_media_session_supports_seekbar: bool,
    pub normal_always_display: bool,
    pub normal_background_blur_style: i32,
    pub media_flyout_acrylic_window_enabled: bool,
    pub exclusive_tidal_mode: bool,
    pub lock_keys_enabled: bool,
    pub lock_keys_insert_enabled: bool,
    pub lock_keys_duration_ms: u64,
    pub lock_keys_bold_ui: bool,
    pub lock_keys_monitor_preference: i32,
    pub lock_keys_acrylic_window_enabled: bool,
    pub settings_path: String,
    pub settings_status: String,
    pub settings_summary: String,
    pub compact_play_pause_label: String,
    pub compact_repeat_label: String,
    pub compact_repeat_active: bool,
    pub compact_repeat_available: bool,
    pub compact_shuffle_active: bool,
    pub compact_shuffle_available: bool,
    pub compact_seekbar_value: f32,
    pub compact_seekbar_maximum: f32,
    pub compact_always_display: bool,
    pub compact_compact_layout: bool,
    pub compact_background_blur_style: i32,
}

pub struct EditableSettings {
    pub media_flyout_enabled: bool,
    pub media_flyout_background_blur: i32,
    pub media_flyout_acrylic_window_enabled: bool,
    pub compact_layout: bool,
    pub position: i32,
    pub duration: i32,
    pub media_flyout_always_display: bool,
    pub center_title_artist: bool,
    pub player_info_enabled: bool,
    pub repeat_enabled: bool,
    pub shuffle_enabled: bool,
    pub seekbar_enabled: bool,
    pub media_flyout_volume_keys_excluded: bool,
    pub exclusive_tidal_mode: bool,
    pub lock_keys_enabled: bool,
    pub lock_keys_acrylic_window_enabled: bool,
    pub lock_keys_duration: i32,
    pub lock_keys_bold_ui: bool,
    pub lock_keys_monitor_preference: i32,
    pub lock_keys_insert_enabled: bool,
    pub flyout_selected_monitor: i32,
    pub startup: bool,
    pub disable_if_fullscreen: bool,
    pub nicon_left_click: i32,
    pub nicon_hide: bool,
    pub app_theme: i32,
    pub next_up_enabled: bool,
    pub next_up_duration: i32,
    pub next_up_acrylic_window_enabled: bool,
    pub taskbar_widget_enabled: bool,
    pub taskbar_widget_selected_monitor: i32,
    pub taskbar_widget_position: i32,
    pub taskbar_widget_padding: bool,
    pub taskbar_widget_manual_padding: i32,
    pub taskbar_widget_clickable: bool,
    pub taskbar_widget_closeable_flyout: bool,
    pub taskbar_widget_background_blur: bool,
    pub taskbar_widget_hide_completely: bool,
    pub taskbar_widget_controls_enabled: bool,
    pub taskbar_widget_controls_position: i32,
    pub taskbar_widget_animated: bool,
}

impl Default for AppPreviewState {
    fn default() -> Self {
        Self::load()
    }
}

impl AppPreviewState {
    pub fn load() -> Self {
        let settings_path = resolve_settings_path();
        let loaded = load_settings_from_disk(&settings_path);
        let settings = loaded.settings.unwrap_or_default();

        let settings_status = match loaded.error {
            None => format!("Loaded {}", settings_path.display()),
            Some(error) => format!("Using defaults: {error}"),
        };

        let settings_summary = format!(
            "compact={}, repeat={}, shuffle={}, seekbar={}, player-info={}, blur={}, acrylic={}, pos={}, lock={}, lock-insert={}, fullscreen-cutoff={}, lang={}, tidal-only={}",
            settings.compact_layout,
            settings.repeat_enabled,
            settings.shuffle_enabled,
            settings.seekbar_enabled,
            settings.player_info_enabled,
            settings.media_flyout_background_blur,
            settings.media_flyout_acrylic_window_enabled,
            settings.position,
            settings.lock_keys_enabled,
            settings.lock_keys_insert_enabled,
            settings.disable_if_fullscreen,
            settings.app_language,
            settings.exclusive_tidal_mode,
        );

        Self {
            normal: MediaFlyoutPreviewState {
                title: "Glass Arcade",
                artist: "Lumen Avenue",
                player: "Spotify",
                current_duration: "1:12",
                max_duration: "3:08",
            },
            compact: MediaFlyoutPreviewState {
                title: "Signal Bloom",
                artist: "North Harbor",
                player: "TIDAL",
                current_duration: "0:48",
                max_duration: "4:21",
            },
            flyout_position: settings.position,
            flyout_selected_monitor: settings.flyout_selected_monitor,
            flyout_animation_speed: settings.flyout_animation_speed,
            flyout_animation_easing_style: settings.flyout_animation_easing_style,
            flyout_duration_ms: settings.duration.clamp(0, 10_000) as u64,
            media_flyout_enabled: settings.media_flyout_enabled,
            media_flyout_volume_keys_excluded: settings.media_flyout_volume_keys_excluded,
            disable_if_fullscreen: settings.disable_if_fullscreen,
            nicon_left_click: settings.nicon_left_click,
            nicon_hide: settings.nicon_hide,
            startup: settings.startup,
            app_theme: settings.app_theme,
            next_up_enabled: settings.next_up_enabled,
            next_up_duration_ms: settings.next_up_duration.clamp(0, 10_000) as u64,
            next_up_acrylic_window_enabled: settings.next_up_acrylic_window_enabled,
            taskbar_widget_enabled: settings.taskbar_widget_enabled,
            taskbar_widget_selected_monitor: settings.taskbar_widget_selected_monitor,
            taskbar_widget_position: settings.taskbar_widget_position,
            taskbar_widget_padding: settings.taskbar_widget_padding,
            taskbar_widget_manual_padding: settings.taskbar_widget_manual_padding,
            taskbar_widget_clickable: settings.taskbar_widget_clickable,
            taskbar_widget_closeable_flyout: settings.taskbar_widget_closeable_flyout,
            taskbar_widget_background_blur: settings.taskbar_widget_background_blur,
            taskbar_widget_hide_completely: settings.taskbar_widget_hide_completely,
            taskbar_widget_controls_enabled: settings.taskbar_widget_controls_enabled,
            taskbar_widget_controls_position: settings.taskbar_widget_controls_position,
            taskbar_widget_animated: settings.taskbar_widget_animated,
            normal_compact_layout: settings.compact_layout,
            normal_center_title_artist: settings.center_title_artist,
            normal_play_pause_label: String::from("II"),
            normal_repeat_label: String::from("R"),
            normal_repeat_active: false,
            normal_repeat_available: true,
            normal_shuffle_active: false,
            normal_shuffle_available: true,
            normal_seekbar_value: 72.0,
            normal_seekbar_maximum: 188.0,
            normal_repeat_enabled: settings.repeat_enabled,
            normal_shuffle_enabled: settings.shuffle_enabled,
            normal_player_info_enabled: settings.player_info_enabled,
            normal_seekbar_enabled: settings.seekbar_enabled,
            normal_media_session_supports_seekbar: true,
            normal_always_display: settings.media_flyout_always_display,
            normal_background_blur_style: settings.media_flyout_background_blur,
            media_flyout_acrylic_window_enabled: settings.media_flyout_acrylic_window_enabled,
            exclusive_tidal_mode: settings.exclusive_tidal_mode,
            lock_keys_enabled: settings.lock_keys_enabled,
            lock_keys_insert_enabled: settings.lock_keys_insert_enabled,
            lock_keys_duration_ms: settings.lock_keys_duration.clamp(0, 10_000) as u64,
            lock_keys_bold_ui: settings.lock_keys_bold_ui,
            lock_keys_monitor_preference: settings.lock_keys_monitor_preference,
            lock_keys_acrylic_window_enabled: settings.lock_keys_acrylic_window_enabled,
            settings_path: settings_path.display().to_string(),
            settings_status,
            settings_summary,
            compact_play_pause_label: String::from("II"),
            compact_repeat_label: String::from("R"),
            compact_repeat_active: false,
            compact_repeat_available: true,
            compact_shuffle_active: false,
            compact_shuffle_available: true,
            compact_seekbar_value: 48.0,
            compact_seekbar_maximum: 261.0,
            compact_always_display: true,
            compact_compact_layout: !settings.compact_layout,
            compact_background_blur_style: if settings.media_flyout_background_blur == 0 {
                2
            } else {
                settings.media_flyout_background_blur
            },
        }
    }
}

struct SettingsLoadResult {
    settings: Option<FluentFlyoutSettings>,
    error: Option<String>,
}

fn load_settings_from_disk(path: &Path) -> SettingsLoadResult {
    match fs::read_to_string(path) {
        Ok(xml) => match quick_xml::de::from_str::<FluentFlyoutSettings>(&xml) {
            Ok(settings) => SettingsLoadResult {
                settings: Some(settings),
                error: None,
            },
            Err(error) => SettingsLoadResult {
                settings: None,
                error: Some(format!("failed to parse XML at {}: {error}", path.display())),
            },
        },
        Err(error) => SettingsLoadResult {
            settings: None,
            error: Some(format!("failed to read {}: {error}", path.display())),
        },
    }
}

fn resolve_settings_path() -> PathBuf {
    if let Some(appdata) = env::var_os("APPDATA") {
        return PathBuf::from(appdata)
            .join("FluentFlyout")
            .join("settings.xml");
    }

    if let Some(user_profile) = env::var_os("USERPROFILE") {
        return PathBuf::from(user_profile)
            .join("AppData")
            .join("Roaming")
            .join("FluentFlyout")
            .join("settings.xml");
    }

    PathBuf::from("/mnt/c/Users/minse/AppData/Roaming/FluentFlyout/settings.xml")
}

pub fn save_settings_edits(edits: &EditableSettings) -> Result<PathBuf, String> {
    let path = resolve_settings_path();
    save_settings_edits_to_path(&path, edits)?;
    Ok(path)
}

fn save_settings_edits_to_path(path: &Path, edits: &EditableSettings) -> Result<(), String> {
    let mut xml = match fs::read_to_string(&path) {
        Ok(existing) => existing,
        Err(_) => String::from(
            "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<UserSettings xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xmlns:xsd=\"http://www.w3.org/2001/XMLSchema\">\n</UserSettings>\n",
        ),
    };

    let fields = [
        ("MediaFlyoutEnabled", bool_text(edits.media_flyout_enabled)),
        (
            "MediaFlyoutBackgroundBlur",
            edits.media_flyout_background_blur.to_string(),
        ),
        (
            "MediaFlyoutAcrylicWindowEnabled",
            bool_text(edits.media_flyout_acrylic_window_enabled),
        ),
        ("CompactLayout", bool_text(edits.compact_layout)),
        ("Position", edits.position.to_string()),
        ("Duration", edits.duration.clamp(0, 10_000).to_string()),
        (
            "MediaFlyoutAlwaysDisplay",
            bool_text(edits.media_flyout_always_display),
        ),
        ("CenterTitleArtist", bool_text(edits.center_title_artist)),
        ("PlayerInfoEnabled", bool_text(edits.player_info_enabled)),
        ("RepeatEnabled", bool_text(edits.repeat_enabled)),
        ("ShuffleEnabled", bool_text(edits.shuffle_enabled)),
        ("SeekbarEnabled", bool_text(edits.seekbar_enabled)),
        (
            "MediaFlyoutVolumeKeysExcluded",
            bool_text(edits.media_flyout_volume_keys_excluded),
        ),
        ("ExclusiveTidalMode", bool_text(edits.exclusive_tidal_mode)),
        ("LockKeysEnabled", bool_text(edits.lock_keys_enabled)),
        (
            "LockKeysAcrylicWindowEnabled",
            bool_text(edits.lock_keys_acrylic_window_enabled),
        ),
        (
            "LockKeysDuration",
            edits.lock_keys_duration.clamp(0, 10_000).to_string(),
        ),
        ("LockKeysBoldUI", bool_text(edits.lock_keys_bold_ui)),
        (
            "LockKeysMonitorPreference",
            edits.lock_keys_monitor_preference.to_string(),
        ),
        (
            "LockKeysInsertEnabled",
            bool_text(edits.lock_keys_insert_enabled),
        ),
        ("FlyoutSelectedMonitor", edits.flyout_selected_monitor.to_string()),
        ("Startup", bool_text(edits.startup)),
        (
            "DisableIfFullscreen",
            bool_text(edits.disable_if_fullscreen),
        ),
        ("nIconLeftClick", edits.nicon_left_click.to_string()),
        ("NIconHide", bool_text(edits.nicon_hide)),
        ("AppTheme", edits.app_theme.to_string()),
        ("NextUpEnabled", bool_text(edits.next_up_enabled)),
        (
            "NextUpDuration",
            edits.next_up_duration.clamp(0, 10_000).to_string(),
        ),
        (
            "NextUpAcrylicWindowEnabled",
            bool_text(edits.next_up_acrylic_window_enabled),
        ),
        ("TaskbarWidgetEnabled", bool_text(edits.taskbar_widget_enabled)),
        (
            "TaskbarWidgetSelectedMonitor",
            edits.taskbar_widget_selected_monitor.to_string(),
        ),
        (
            "TaskbarWidgetPosition",
            edits.taskbar_widget_position.to_string(),
        ),
        ("TaskbarWidgetPadding", bool_text(edits.taskbar_widget_padding)),
        (
            "TaskbarWidgetManualPadding",
            edits.taskbar_widget_manual_padding.to_string(),
        ),
        (
            "TaskbarWidgetClickable",
            bool_text(edits.taskbar_widget_clickable),
        ),
        (
            "TaskbarWidgetCloseableFlyout",
            bool_text(edits.taskbar_widget_closeable_flyout),
        ),
        (
            "TaskbarWidgetBackgroundBlur",
            bool_text(edits.taskbar_widget_background_blur),
        ),
        (
            "TaskbarWidgetHideCompletely",
            bool_text(edits.taskbar_widget_hide_completely),
        ),
        (
            "TaskbarWidgetControlsEnabled",
            bool_text(edits.taskbar_widget_controls_enabled),
        ),
        (
            "TaskbarWidgetControlsPosition",
            edits.taskbar_widget_controls_position.to_string(),
        ),
        (
            "TaskbarWidgetAnimated",
            bool_text(edits.taskbar_widget_animated),
        ),
    ];

    for (tag, value) in fields {
        replace_or_insert_tag(&mut xml, tag, &value);
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
    }

    fs::write(path, xml).map_err(|error| format!("failed to write {}: {error}", path.display()))?;
    Ok(())
}

fn bool_text(value: bool) -> String {
    if value {
        String::from("true")
    } else {
        String::from("false")
    }
}

fn replace_or_insert_tag(xml: &mut String, tag: &str, value: &str) {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");

    if let Some(start) = xml.find(&open) {
        let content_start = start + open.len();
        if let Some(end_relative) = xml[content_start..].find(&close) {
            let end = content_start + end_relative;
            xml.replace_range(content_start..end, value);
            return;
        }
    }

    if let Some(end_root) = xml.rfind("</UserSettings>") {
        let insertion = format!("  {open}{value}{close}\n");
        xml.insert_str(end_root, &insertion);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static TEST_COUNTER: AtomicUsize = AtomicUsize::new(0);

    #[test]
    fn load_settings_from_disk_parses_current_tags() {
        let path = temp_settings_path("parse");
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<UserSettings xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:xsd="http://www.w3.org/2001/XMLSchema">
  <CompactLayout>true</CompactLayout>
  <FlyoutSelectedMonitor>2</FlyoutSelectedMonitor>
  <Position>5</Position>
  <Duration>4500</Duration>
  <MediaFlyoutEnabled>false</MediaFlyoutEnabled>
  <MediaFlyoutVolumeKeysExcluded>true</MediaFlyoutVolumeKeysExcluded>
  <nIconLeftClick>1</nIconLeftClick>
  <NIconHide>true</NIconHide>
  <Startup>false</Startup>
  <AppTheme>2</AppTheme>
  <LockKeysAcrylicWindowEnabled>false</LockKeysAcrylicWindowEnabled>
  <NextUpEnabled>false</NextUpEnabled>
  <NextUpDuration>3500</NextUpDuration>
  <TaskbarWidgetEnabled>false</TaskbarWidgetEnabled>
  <TaskbarWidgetPosition>2</TaskbarWidgetPosition>
  <TaskbarWidgetControlsPosition>0</TaskbarWidgetControlsPosition>
</UserSettings>
"#;

        write_test_file(&path, xml);
        let loaded = load_settings_from_disk(&path);
        let settings = loaded.settings.expect("settings should parse");

        assert!(loaded.error.is_none());
        assert!(settings.compact_layout);
        assert_eq!(settings.flyout_selected_monitor, 2);
        assert_eq!(settings.position, 5);
        assert_eq!(settings.duration, 4500);
        assert!(!settings.media_flyout_enabled);
        assert!(settings.media_flyout_volume_keys_excluded);
        assert_eq!(settings.nicon_left_click, 1);
        assert!(settings.nicon_hide);
        assert!(!settings.startup);
        assert_eq!(settings.app_theme, 2);
        assert!(!settings.lock_keys_acrylic_window_enabled);
        assert!(!settings.next_up_enabled);
        assert_eq!(settings.next_up_duration, 3500);
        assert!(!settings.taskbar_widget_enabled);
        assert_eq!(settings.taskbar_widget_position, 2);
        assert_eq!(settings.taskbar_widget_controls_position, 0);

        cleanup_test_file(&path);
    }

    #[test]
    fn save_settings_edits_updates_known_tags_and_preserves_unknown_ones() {
        let path = temp_settings_path("save");
        let initial_xml = r#"<?xml version="1.0" encoding="utf-8"?>
<UserSettings xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:xsd="http://www.w3.org/2001/XMLSchema">
  <CompactLayout>false</CompactLayout>
  <Duration>3000</Duration>
  <NIconHide>false</NIconHide>
  <LastKnownVersion>debug</LastKnownVersion>
</UserSettings>
"#;
        write_test_file(&path, initial_xml);

        let edits = sample_edits();
        save_settings_edits_to_path(&path, &edits).expect("save should succeed");

        let saved_xml = fs::read_to_string(&path).expect("saved xml should exist");
        assert!(saved_xml.contains("<CompactLayout>true</CompactLayout>"));
        assert!(saved_xml.contains("<Duration>10000</Duration>"));
        assert!(saved_xml.contains("<NIconHide>true</NIconHide>"));
        assert!(saved_xml.contains("<NextUpEnabled>false</NextUpEnabled>"));
        assert!(saved_xml.contains("<TaskbarWidgetManualPadding>24</TaskbarWidgetManualPadding>"));
        assert!(saved_xml.contains("<LastKnownVersion>debug</LastKnownVersion>"));

        let reparsed = quick_xml::de::from_str::<FluentFlyoutSettings>(&saved_xml)
            .expect("saved xml should remain parseable");
        assert!(reparsed.compact_layout);
        assert_eq!(reparsed.duration, 10_000);
        assert!(reparsed.nicon_hide);
        assert!(!reparsed.next_up_enabled);
        assert_eq!(reparsed.taskbar_widget_manual_padding, 24);

        cleanup_test_file(&path);
    }

    #[test]
    fn replace_or_insert_tag_inserts_missing_tag_before_root_end() {
        let mut xml = String::from(
            "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<UserSettings>\n  <CompactLayout>false</CompactLayout>\n</UserSettings>\n",
        );

        replace_or_insert_tag(&mut xml, "TaskbarWidgetEnabled", "true");
        replace_or_insert_tag(&mut xml, "CompactLayout", "true");

        assert!(xml.contains("<CompactLayout>true</CompactLayout>"));
        assert!(xml.contains("<TaskbarWidgetEnabled>true</TaskbarWidgetEnabled>"));
        assert!(xml.find("<TaskbarWidgetEnabled>true</TaskbarWidgetEnabled>").unwrap()
            < xml.find("</UserSettings>").unwrap());
    }

    fn sample_edits() -> EditableSettings {
        EditableSettings {
            media_flyout_enabled: false,
            media_flyout_background_blur: 3,
            media_flyout_acrylic_window_enabled: false,
            compact_layout: true,
            position: 5,
            duration: 20_000,
            media_flyout_always_display: true,
            center_title_artist: true,
            player_info_enabled: false,
            repeat_enabled: true,
            shuffle_enabled: true,
            seekbar_enabled: true,
            media_flyout_volume_keys_excluded: true,
            exclusive_tidal_mode: false,
            lock_keys_enabled: false,
            lock_keys_acrylic_window_enabled: false,
            lock_keys_duration: 11_000,
            lock_keys_bold_ui: true,
            lock_keys_monitor_preference: 2,
            lock_keys_insert_enabled: false,
            flyout_selected_monitor: 4,
            startup: false,
            disable_if_fullscreen: false,
            nicon_left_click: 1,
            nicon_hide: true,
            app_theme: 2,
            next_up_enabled: false,
            next_up_duration: 8_000,
            next_up_acrylic_window_enabled: false,
            taskbar_widget_enabled: false,
            taskbar_widget_selected_monitor: 3,
            taskbar_widget_position: 2,
            taskbar_widget_padding: false,
            taskbar_widget_manual_padding: 24,
            taskbar_widget_clickable: false,
            taskbar_widget_closeable_flyout: false,
            taskbar_widget_background_blur: true,
            taskbar_widget_hide_completely: true,
            taskbar_widget_controls_enabled: false,
            taskbar_widget_controls_position: 0,
            taskbar_widget_animated: false,
        }
    }

    fn temp_settings_path(label: &str) -> PathBuf {
        let sequence = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic enough for tests")
            .as_nanos();
        env::temp_dir()
            .join("fluent_flyout_rs_state_tests")
            .join(format!("{label}-{nanos}-{sequence}"))
            .join("settings.xml")
    }

    fn write_test_file(path: &Path, xml: &str) {
        fs::create_dir_all(path.parent().expect("test path should have parent"))
            .expect("test directory should be created");
        fs::write(path, xml).expect("test xml should be written");
    }

    fn cleanup_test_file(path: &Path) {
        if let Some(root) = path.parent().and_then(|parent| parent.parent()) {
            let _ = fs::remove_dir_all(root);
        }
    }
}
