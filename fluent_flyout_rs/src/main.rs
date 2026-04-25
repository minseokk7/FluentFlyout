#![windows_subsystem = "windows"]

mod media;
mod monitor;
mod state;
mod taskbar_native;

use fluent_flyout_core::{
    get_keyboard_action_flags, is_fullscreen_app_running, set_keyboard_hook, unset_keyboard_hook,
    update_input_config,
};
use image::ImageReader;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use slint::{ComponentHandle, Image, PhysicalPosition};
use std::env;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{mpsc, Arc, Mutex, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{
    Icon as TrayIconImage, MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder,
    TrayIconEvent,
};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Input::KeyboardAndMouse::GetKeyState;

slint::include_modules!();

const INPUT_ACTION_SHOW_MEDIA_FLYOUT: u32 = 1 << 0;
const INPUT_ACTION_LOCK_CAPS: u32 = 1 << 1;
const INPUT_ACTION_LOCK_NUM: u32 = 1 << 2;
const INPUT_ACTION_LOCK_SCROLL: u32 = 1 << 3;
const INPUT_ACTION_LOCK_INSERT: u32 = 1 << 4;

static KEYBOARD_EVENT_SENDER: OnceLock<mpsc::Sender<i32>> = OnceLock::new();

#[derive(Default)]
struct RuntimeState {
    last_session_key: Mutex<Option<String>>,
    media_visibility_epoch: AtomicU64,
    lock_visibility_epoch: AtomicU64,
    last_media_input_ms: AtomicU64,
}

#[derive(Clone, Copy)]
struct RuntimeConfig {
    exclusive_tidal_mode: bool,
    flyout_position: i32,
    flyout_selected_monitor: i32,
    flyout_duration_ms: u64,
    media_flyout_enabled: bool,
    keep_visible_when_idle: bool,
    disable_if_fullscreen: bool,
    nicon_left_click: i32,
    nicon_hide: bool,
    lock_keys_enabled: bool,
    lock_keys_duration_ms: u64,
    lock_keys_bold_ui: bool,
    lock_keys_monitor_preference: i32,
    taskbar_widget_enabled: bool,
    taskbar_widget_selected_monitor: i32,
    taskbar_widget_position: i32,
    taskbar_widget_padding: bool,
    taskbar_widget_manual_padding: i32,
    taskbar_widget_clickable: bool,
    taskbar_widget_hide_completely: bool,
    taskbar_widget_controls_enabled: bool,
    taskbar_widget_controls_position: i32,
    taskbar_widget_background_blur: bool,
}

fn main() -> Result<(), slint::PlatformError> {
    configure_slint_backend();

    let app = AppWindow::new()?;
    let live_flyout = LiveFlyoutWindow::new()?;
    let taskbar_widget = TaskbarWidgetWindow::new()?;
    let lock_flyout = LockFlyoutWindow::new()?;
    let settings_window = SettingsWindow::new()?;
    let preview = state::AppPreviewState::default();
    let runtime_state = Arc::new(RuntimeState::default());
    let runtime_config = config_from_preview(&preview);

    let app_weak = app.as_weak();
    let live_flyout_weak = live_flyout.as_weak();
    let taskbar_widget_weak = taskbar_widget.as_weak();
    let lock_flyout_weak = lock_flyout.as_weak();
    let settings_window_weak = settings_window.as_weak();

    apply_dashboard_preview(&app, &preview);
    apply_live_flyout_preview(&live_flyout, &preview);
    apply_taskbar_widget_preview(&taskbar_widget, &preview);
    apply_lock_flyout_preview(&lock_flyout, &preview);
    apply_settings_window_preview(&settings_window, &preview, None);
    wire_media_controls(&app, runtime_config.exclusive_tidal_mode);
    wire_live_flyout_controls(
        &live_flyout,
        runtime_config.exclusive_tidal_mode,
        runtime_state.clone(),
    );
    wire_taskbar_widget_controls(
        &taskbar_widget,
        app_weak.clone(),
        live_flyout_weak.clone(),
        taskbar_widget_weak.clone(),
        runtime_state.clone(),
        runtime_config,
    );
    wire_settings_window_controls(
        &settings_window,
        app_weak.clone(),
        live_flyout_weak.clone(),
        taskbar_widget_weak.clone(),
        lock_flyout_weak.clone(),
        runtime_state.clone(),
    );

    settings_window.show()?;
    if runtime_config.media_flyout_enabled
        && runtime_config.keep_visible_when_idle
        && !runtime_config.disable_if_fullscreen
    {
        live_flyout.show()?;
        apply_live_flyout_position(
            &live_flyout,
            runtime_config.flyout_position,
            runtime_config.flyout_selected_monitor,
        );
    }
    if runtime_config.taskbar_widget_enabled && !runtime_config.taskbar_widget_hide_completely {
        taskbar_widget.show()?;
        apply_taskbar_widget_position(&taskbar_widget, runtime_config);
    }

    refresh_live_media(
        app_weak.clone(),
        live_flyout_weak.clone(),
        taskbar_widget_weak.clone(),
        runtime_state.clone(),
        runtime_config,
        false,
    );

    start_keyboard_hook_worker(
        app_weak.clone(),
        live_flyout_weak.clone(),
        taskbar_widget_weak.clone(),
        lock_flyout_weak.clone(),
        runtime_state.clone(),
        runtime_config,
        preview.lock_keys_insert_enabled,
        preview.media_flyout_volume_keys_excluded,
    );

    let mut tray_resources = None;
    if !runtime_config.nicon_hide {
        tray_resources = setup_tray_icon(
            app_weak.clone(),
            settings_window_weak.clone(),
            live_flyout_weak.clone(),
            taskbar_widget_weak.clone(),
            runtime_state.clone(),
            runtime_config,
        );
    }

    let poll_timer = slint::Timer::default();
    poll_timer.start(
        slint::TimerMode::Repeated,
        Duration::from_secs(2),
        move || {
            refresh_live_media(
                app_weak.clone(),
                live_flyout_weak.clone(),
                taskbar_widget_weak.clone(),
                runtime_state.clone(),
                runtime_config,
                false,
            )
        },
    );

    let run_result = slint::run_event_loop();
    unset_keyboard_hook();
    drop(tray_resources);
    run_result
}

fn configure_slint_backend() {
    if env::var_os("SLINT_BACKEND").is_none() {
        // Prefer the software renderer so the app can start in environments
        // without working OpenGL drivers.
        unsafe {
            env::set_var("SLINT_BACKEND", "software");
        }
    }
}

fn config_from_preview(preview: &state::AppPreviewState) -> RuntimeConfig {
    RuntimeConfig {
        exclusive_tidal_mode: preview.exclusive_tidal_mode,
        flyout_position: preview.flyout_position,
        flyout_selected_monitor: preview.flyout_selected_monitor,
        flyout_duration_ms: preview
            .flyout_duration_ms
            .saturating_add(animation_tail_ms(
                preview.flyout_animation_speed,
                preview.flyout_animation_easing_style,
            )),
        media_flyout_enabled: preview.media_flyout_enabled,
        keep_visible_when_idle: preview.normal_always_display,
        disable_if_fullscreen: preview.disable_if_fullscreen,
        nicon_left_click: preview.nicon_left_click,
        nicon_hide: preview.nicon_hide,
        lock_keys_enabled: preview.lock_keys_enabled,
        lock_keys_duration_ms: preview
            .lock_keys_duration_ms
            .saturating_add(animation_tail_ms(
                preview.flyout_animation_speed,
                preview.flyout_animation_easing_style,
            )),
        lock_keys_bold_ui: preview.lock_keys_bold_ui,
        lock_keys_monitor_preference: preview.lock_keys_monitor_preference,
        taskbar_widget_enabled: preview.taskbar_widget_enabled,
        taskbar_widget_selected_monitor: preview.taskbar_widget_selected_monitor,
        taskbar_widget_position: preview.taskbar_widget_position,
        taskbar_widget_padding: preview.taskbar_widget_padding,
        taskbar_widget_manual_padding: preview.taskbar_widget_manual_padding,
        taskbar_widget_clickable: preview.taskbar_widget_clickable,
        taskbar_widget_hide_completely: preview.taskbar_widget_hide_completely,
        taskbar_widget_controls_enabled: preview.taskbar_widget_controls_enabled,
        taskbar_widget_controls_position: preview.taskbar_widget_controls_position,
        taskbar_widget_background_blur: preview.taskbar_widget_background_blur,
    }
}

fn apply_dashboard_preview(app: &AppWindow, preview: &state::AppPreviewState) {
    app.set_normalCompactLayout(preview.normal_compact_layout);
    app.set_normalCenterTitleArtist(preview.normal_center_title_artist);
    app.set_normalPlayPauseLabel(preview.normal_play_pause_label.clone().into());
    app.set_normalRepeatLabel(preview.normal_repeat_label.clone().into());
    app.set_normalRepeatActive(preview.normal_repeat_active);
    app.set_normalRepeatAvailable(preview.normal_repeat_available);
    app.set_normalShuffleActive(preview.normal_shuffle_active);
    app.set_normalShuffleAvailable(preview.normal_shuffle_available);
    app.set_normalTitle(preview.normal.title.into());
    app.set_normalArtist(preview.normal.artist.into());
    app.set_normalPlayer(preview.normal.player.into());
    app.set_normalCurrentDuration(preview.normal.current_duration.into());
    app.set_normalMaxDuration(preview.normal.max_duration.into());
    app.set_normalAlbumArt(Image::default());
    app.set_normalHasAlbumArt(false);
    app.set_normalSeekbarValue(preview.normal_seekbar_value);
    app.set_normalSeekbarMaximum(preview.normal_seekbar_maximum);
    app.set_normalRepeatEnabled(preview.normal_repeat_enabled);
    app.set_normalShuffleEnabled(preview.normal_shuffle_enabled);
    app.set_normalPlayerInfoEnabled(preview.normal_player_info_enabled);
    app.set_normalSeekbarEnabled(preview.normal_seekbar_enabled);
    app.set_normalMediaSessionSupportsSeekbar(preview.normal_media_session_supports_seekbar);
    app.set_normalAlwaysDisplay(preview.normal_always_display);
    app.set_normalBackgroundBlurStyle(preview.normal_background_blur_style);
    app.set_settingsPath(preview.settings_path.clone().into());
    app.set_settingsStatus(preview.settings_status.clone().into());
    app.set_settingsSummary(preview.settings_summary.clone().into());

    app.set_compactPlayPauseLabel(preview.compact_play_pause_label.clone().into());
    app.set_compactRepeatLabel(preview.compact_repeat_label.clone().into());
    app.set_compactRepeatActive(preview.compact_repeat_active);
    app.set_compactRepeatAvailable(preview.compact_repeat_available);
    app.set_compactShuffleActive(preview.compact_shuffle_active);
    app.set_compactShuffleAvailable(preview.compact_shuffle_available);
    app.set_compactTitle(preview.compact.title.into());
    app.set_compactArtist(preview.compact.artist.into());
    app.set_compactPlayer(preview.compact.player.into());
    app.set_compactCurrentDuration(preview.compact.current_duration.into());
    app.set_compactMaxDuration(preview.compact.max_duration.into());
    app.set_compactAlbumArt(Image::default());
    app.set_compactHasAlbumArt(false);
    app.set_compactSeekbarValue(preview.compact_seekbar_value);
    app.set_compactSeekbarMaximum(preview.compact_seekbar_maximum);
    app.set_compactAlwaysDisplay(preview.compact_always_display);
    app.set_compactCompactLayout(preview.compact_compact_layout);
    app.set_compactBackgroundBlurStyle(preview.compact_background_blur_style);
}

fn apply_live_flyout_preview(live_flyout: &LiveFlyoutWindow, preview: &state::AppPreviewState) {
    live_flyout.set_compactLayout(preview.normal_compact_layout);
    live_flyout.set_centerTitleArtist(preview.normal_center_title_artist);
    live_flyout.set_playPauseLabel(preview.normal_play_pause_label.clone().into());
    live_flyout.set_repeatLabel(preview.normal_repeat_label.clone().into());
    live_flyout.set_repeatActive(preview.normal_repeat_active);
    live_flyout.set_repeatAvailable(preview.normal_repeat_available);
    live_flyout.set_shuffleActive(preview.normal_shuffle_active);
    live_flyout.set_shuffleAvailable(preview.normal_shuffle_available);
    live_flyout.set_titleText(preview.normal.title.into());
    live_flyout.set_artistText(preview.normal.artist.into());
    live_flyout.set_playerText(preview.normal.player.into());
    live_flyout.set_currentDurationText(preview.normal.current_duration.into());
    live_flyout.set_maxDurationText(preview.normal.max_duration.into());
    live_flyout.set_albumArt(Image::default());
    live_flyout.set_hasAlbumArt(false);
    live_flyout.set_seekbarValue(preview.normal_seekbar_value);
    live_flyout.set_seekbarMaximum(preview.normal_seekbar_maximum);
    live_flyout.set_repeatEnabled(preview.normal_repeat_enabled);
    live_flyout.set_shuffleEnabled(preview.normal_shuffle_enabled);
    live_flyout.set_playerInfoEnabled(preview.normal_player_info_enabled);
    live_flyout.set_seekbarEnabled(preview.normal_seekbar_enabled);
    live_flyout.set_mediaSessionSupportsSeekbar(preview.normal_media_session_supports_seekbar);
    live_flyout.set_mediaFlyoutAlwaysDisplay(preview.normal_always_display);
    live_flyout.set_backgroundBlurStyle(preview.normal_background_blur_style);
}

fn apply_taskbar_widget_preview(
    taskbar_widget: &TaskbarWidgetWindow,
    preview: &state::AppPreviewState,
) {
    taskbar_widget.set_titleText(preview.normal.title.into());
    taskbar_widget.set_artistText(preview.normal.artist.into());
    taskbar_widget.set_playerText(preview.normal.player.into());
    taskbar_widget.set_albumArt(Image::default());
    taskbar_widget.set_hasAlbumArt(false);
    taskbar_widget.set_hasMedia(true);
    taskbar_widget.set_isPlaying(true);
    taskbar_widget.set_playPauseLabel(preview.normal_play_pause_label.clone().into());
    taskbar_widget.set_controlsEnabled(preview.taskbar_widget_controls_enabled);
    taskbar_widget.set_controlsPosition(preview.taskbar_widget_controls_position);
    taskbar_widget.set_backgroundBlur(preview.taskbar_widget_background_blur);
}

fn apply_lock_flyout_preview(lock_flyout: &LockFlyoutWindow, preview: &state::AppPreviewState) {
    lock_flyout.set_titleText("Caps Lock On".into());
    lock_flyout.set_statusOn(true);
    lock_flyout.set_boldUi(preview.lock_keys_bold_ui);
}

fn apply_settings_window_preview(
    settings_window: &SettingsWindow,
    preview: &state::AppPreviewState,
    save_status: Option<String>,
) {
    settings_window.set_settingsPath(preview.settings_path.clone().into());
    settings_window.set_saveStatus(
        save_status
            .unwrap_or_else(|| String::from("Ready"))
            .into(),
    );
    settings_window.set_mediaFlyoutEnabled(preview.media_flyout_enabled);
    settings_window.set_mediaFlyoutBackgroundBlur(preview.normal_background_blur_style);
    settings_window.set_mediaFlyoutAcrylicWindowEnabled(preview.media_flyout_acrylic_window_enabled);
    settings_window.set_compactLayout(preview.normal_compact_layout);
    settings_window.set_position(preview.flyout_position);
    settings_window.set_durationMs(preview.flyout_duration_ms as i32);
    settings_window.set_mediaFlyoutAlwaysDisplay(preview.normal_always_display);
    settings_window.set_centerTitleArtist(preview.normal_center_title_artist);
    settings_window.set_playerInfoEnabled(preview.normal_player_info_enabled);
    settings_window.set_repeatEnabled(preview.normal_repeat_enabled);
    settings_window.set_shuffleEnabled(preview.normal_shuffle_enabled);
    settings_window.set_seekbarEnabled(preview.normal_seekbar_enabled);
    settings_window
        .set_mediaFlyoutVolumeKeysExcluded(preview.media_flyout_volume_keys_excluded);
    settings_window.set_exclusiveTidalMode(preview.exclusive_tidal_mode);
    settings_window.set_lockKeysEnabled(preview.lock_keys_enabled);
    settings_window.set_lockKeysAcrylicWindowEnabled(preview.lock_keys_acrylic_window_enabled);
    settings_window.set_lockKeysDurationMs(preview.lock_keys_duration_ms as i32);
    settings_window.set_lockKeysBoldUi(preview.lock_keys_bold_ui);
    settings_window.set_lockKeysMonitorPreference(preview.lock_keys_monitor_preference);
    settings_window.set_lockKeysInsertEnabled(preview.lock_keys_insert_enabled);
    settings_window.set_flyoutSelectedMonitor(preview.flyout_selected_monitor);
    settings_window.set_startup(preview.startup);
    settings_window.set_disableIfFullscreen(preview.disable_if_fullscreen);
    settings_window.set_niconLeftClick(preview.nicon_left_click);
    settings_window.set_niconHide(preview.nicon_hide);
    settings_window.set_appTheme(preview.app_theme);
    settings_window.set_nextUpEnabled(preview.next_up_enabled);
    settings_window.set_nextUpDurationMs(preview.next_up_duration_ms as i32);
    settings_window.set_nextUpAcrylicWindowEnabled(preview.next_up_acrylic_window_enabled);
    settings_window.set_taskbarWidgetEnabled(preview.taskbar_widget_enabled);
    settings_window.set_taskbarWidgetSelectedMonitor(preview.taskbar_widget_selected_monitor);
    settings_window.set_taskbarWidgetPosition(preview.taskbar_widget_position);
    settings_window.set_taskbarWidgetPadding(preview.taskbar_widget_padding);
    settings_window.set_taskbarWidgetManualPadding(preview.taskbar_widget_manual_padding);
    settings_window.set_taskbarWidgetClickable(preview.taskbar_widget_clickable);
    settings_window.set_taskbarWidgetCloseableFlyout(preview.taskbar_widget_closeable_flyout);
    settings_window.set_taskbarWidgetBackgroundBlur(preview.taskbar_widget_background_blur);
    settings_window.set_taskbarWidgetHideCompletely(preview.taskbar_widget_hide_completely);
    settings_window.set_taskbarWidgetControlsEnabled(preview.taskbar_widget_controls_enabled);
    settings_window.set_taskbarWidgetControlsPosition(preview.taskbar_widget_controls_position);
    settings_window.set_taskbarWidgetAnimated(preview.taskbar_widget_animated);
}

fn wire_media_controls(app: &AppWindow, exclusive_tidal_mode: bool) {
    app.on_normalPreviousRequested(move || {
        run_media_action(exclusive_tidal_mode, media::skip_previous);
    });

    app.on_normalPlayPauseRequested(move || {
        run_media_action(exclusive_tidal_mode, media::toggle_play_pause);
    });

    app.on_normalNextRequested(move || {
        run_media_action(exclusive_tidal_mode, media::skip_next);
    });

    app.on_normalRepeatRequested(move || {
        run_media_action(exclusive_tidal_mode, media::cycle_repeat_mode);
    });

    app.on_normalShuffleRequested(move || {
        run_media_action(exclusive_tidal_mode, media::toggle_shuffle);
    });

    app.on_normalCloseRequested(move || {
        let _ = slint::quit_event_loop();
    });

    app.on_normalSeekbarReleased(move |value| {
        run_seek_action(exclusive_tidal_mode, value);
    });
}

fn wire_live_flyout_controls(
    live_flyout: &LiveFlyoutWindow,
    exclusive_tidal_mode: bool,
    runtime_state: Arc<RuntimeState>,
) {
    live_flyout.on_previousRequested(move || {
        run_media_action(exclusive_tidal_mode, media::skip_previous);
    });

    live_flyout.on_playPauseRequested(move || {
        run_media_action(exclusive_tidal_mode, media::toggle_play_pause);
    });

    live_flyout.on_nextRequested(move || {
        run_media_action(exclusive_tidal_mode, media::skip_next);
    });

    live_flyout.on_repeatRequested(move || {
        run_media_action(exclusive_tidal_mode, media::cycle_repeat_mode);
    });

    live_flyout.on_shuffleRequested(move || {
        run_media_action(exclusive_tidal_mode, media::toggle_shuffle);
    });

    let flyout_weak = live_flyout.as_weak();
    live_flyout.on_closeRequested(move || {
        bump_media_visibility_epoch(&runtime_state);
        if let Some(flyout) = flyout_weak.upgrade() {
            let _ = flyout.hide();
        }
    });

    live_flyout.on_seekbarReleased(move |value| {
        run_seek_action(exclusive_tidal_mode, value);
    });
}

fn wire_taskbar_widget_controls(
    taskbar_widget: &TaskbarWidgetWindow,
    app_weak: slint::Weak<AppWindow>,
    live_flyout_weak: slint::Weak<LiveFlyoutWindow>,
    taskbar_widget_weak: slint::Weak<TaskbarWidgetWindow>,
    runtime_state: Arc<RuntimeState>,
    runtime_config: RuntimeConfig,
) {
    taskbar_widget.on_previousRequested(move || {
        run_media_action(runtime_config.exclusive_tidal_mode, media::skip_previous);
    });

    taskbar_widget.on_playPauseRequested(move || {
        run_media_action(runtime_config.exclusive_tidal_mode, media::toggle_play_pause);
    });

    taskbar_widget.on_nextRequested(move || {
        run_media_action(runtime_config.exclusive_tidal_mode, media::skip_next);
    });

    taskbar_widget.on_widgetClicked(move || {
        if runtime_config.taskbar_widget_clickable {
            refresh_live_media(
                app_weak.clone(),
                live_flyout_weak.clone(),
                taskbar_widget_weak.clone(),
                runtime_state.clone(),
                runtime_config,
                true,
            );
        }
    });
}

fn wire_settings_window_controls(
    settings_window: &SettingsWindow,
    app_weak: slint::Weak<AppWindow>,
    live_flyout_weak: slint::Weak<LiveFlyoutWindow>,
    taskbar_widget_weak: slint::Weak<TaskbarWidgetWindow>,
    lock_flyout_weak: slint::Weak<LockFlyoutWindow>,
    runtime_state: Arc<RuntimeState>,
) {
    let settings_weak = settings_window.as_weak();
    settings_window.on_closeRequested(move || {
        if let Some(window) = settings_weak.upgrade() {
            let _ = window.hide();
        }
    });

    let settings_weak = settings_window.as_weak();
    settings_window.on_saveRequested(move || {
        let Some(window) = settings_weak.upgrade() else {
            return;
        };

        let edits = state::EditableSettings {
            media_flyout_enabled: window.get_mediaFlyoutEnabled(),
            media_flyout_background_blur: window.get_mediaFlyoutBackgroundBlur(),
            media_flyout_acrylic_window_enabled: window.get_mediaFlyoutAcrylicWindowEnabled(),
            compact_layout: window.get_compactLayout(),
            position: window.get_position(),
            duration: window.get_durationMs(),
            media_flyout_always_display: window.get_mediaFlyoutAlwaysDisplay(),
            center_title_artist: window.get_centerTitleArtist(),
            player_info_enabled: window.get_playerInfoEnabled(),
            repeat_enabled: window.get_repeatEnabled(),
            shuffle_enabled: window.get_shuffleEnabled(),
            seekbar_enabled: window.get_seekbarEnabled(),
            media_flyout_volume_keys_excluded: window.get_mediaFlyoutVolumeKeysExcluded(),
            exclusive_tidal_mode: window.get_exclusiveTidalMode(),
            lock_keys_enabled: window.get_lockKeysEnabled(),
            lock_keys_acrylic_window_enabled: window.get_lockKeysAcrylicWindowEnabled(),
            lock_keys_duration: window.get_lockKeysDurationMs(),
            lock_keys_bold_ui: window.get_lockKeysBoldUi(),
            lock_keys_monitor_preference: window.get_lockKeysMonitorPreference(),
            lock_keys_insert_enabled: window.get_lockKeysInsertEnabled(),
            flyout_selected_monitor: window.get_flyoutSelectedMonitor(),
            startup: window.get_startup(),
            disable_if_fullscreen: window.get_disableIfFullscreen(),
            nicon_left_click: window.get_niconLeftClick(),
            nicon_hide: window.get_niconHide(),
            app_theme: window.get_appTheme(),
            next_up_enabled: window.get_nextUpEnabled(),
            next_up_duration: window.get_nextUpDurationMs(),
            next_up_acrylic_window_enabled: window.get_nextUpAcrylicWindowEnabled(),
            taskbar_widget_enabled: window.get_taskbarWidgetEnabled(),
            taskbar_widget_selected_monitor: window.get_taskbarWidgetSelectedMonitor(),
            taskbar_widget_position: window.get_taskbarWidgetPosition(),
            taskbar_widget_padding: window.get_taskbarWidgetPadding(),
            taskbar_widget_manual_padding: window.get_taskbarWidgetManualPadding(),
            taskbar_widget_clickable: window.get_taskbarWidgetClickable(),
            taskbar_widget_closeable_flyout: window.get_taskbarWidgetCloseableFlyout(),
            taskbar_widget_background_blur: window.get_taskbarWidgetBackgroundBlur(),
            taskbar_widget_hide_completely: window.get_taskbarWidgetHideCompletely(),
            taskbar_widget_controls_enabled: window.get_taskbarWidgetControlsEnabled(),
            taskbar_widget_controls_position: window.get_taskbarWidgetControlsPosition(),
            taskbar_widget_animated: window.get_taskbarWidgetAnimated(),
        };

        let save_message = match state::save_settings_edits(&edits) {
            Ok(path) => format!("Saved {}", path.display()),
            Err(error) => format!("Save failed: {error}"),
        };

        let preview = state::AppPreviewState::load();
        if let Some(app) = app_weak.upgrade() {
            apply_dashboard_preview(&app, &preview);
        }
        if let Some(live_flyout) = live_flyout_weak.upgrade() {
            apply_live_flyout_preview(&live_flyout, &preview);
        }
        if let Some(taskbar_widget) = taskbar_widget_weak.upgrade() {
            let runtime_config = config_from_preview(&preview);
            apply_taskbar_widget_preview(&taskbar_widget, &preview);
            if runtime_config.taskbar_widget_enabled && !runtime_config.taskbar_widget_hide_completely
            {
                let _ = taskbar_widget.show();
                apply_taskbar_widget_position(&taskbar_widget, runtime_config);
            } else {
                let _ = taskbar_widget.hide();
            }
        }
        if let Some(lock_flyout) = lock_flyout_weak.upgrade() {
            apply_lock_flyout_preview(&lock_flyout, &preview);
        }
        apply_settings_window_preview(&window, &preview, Some(save_message));

        update_input_config(
            preview.media_flyout_enabled,
            preview.media_flyout_volume_keys_excluded,
            preview.lock_keys_enabled,
            preview.lock_keys_insert_enabled,
        );

        refresh_live_media(
            app_weak.clone(),
            live_flyout_weak.clone(),
            taskbar_widget_weak.clone(),
            runtime_state.clone(),
            config_from_preview(&preview),
            true,
        );
    });
}

fn start_keyboard_hook_worker(
    app_weak: slint::Weak<AppWindow>,
    live_flyout_weak: slint::Weak<LiveFlyoutWindow>,
    taskbar_widget_weak: slint::Weak<TaskbarWidgetWindow>,
    lock_flyout_weak: slint::Weak<LockFlyoutWindow>,
    runtime_state: Arc<RuntimeState>,
    runtime_config: RuntimeConfig,
    lock_keys_insert_enabled: bool,
    media_flyout_volume_keys_excluded: bool,
) {
    let (tx, rx) = mpsc::channel::<i32>();
    let _ = KEYBOARD_EVENT_SENDER.set(tx);

    update_input_config(
        runtime_config.media_flyout_enabled,
        media_flyout_volume_keys_excluded,
        runtime_config.lock_keys_enabled,
        lock_keys_insert_enabled,
    );

    let _ = set_keyboard_hook(keyboard_hook_callback);

    std::thread::spawn(move || {
        while let Ok(vk_code) = rx.recv() {
            let is_fullscreen_blocked =
                runtime_config.disable_if_fullscreen && is_fullscreen_app_running();
            let flags = get_keyboard_action_flags(vk_code);

            if !is_fullscreen_blocked
                && (flags & INPUT_ACTION_SHOW_MEDIA_FLYOUT) != 0
                && should_debounce_media_trigger(&runtime_state)
            {
                refresh_live_media(
                    app_weak.clone(),
                    live_flyout_weak.clone(),
                    taskbar_widget_weak.clone(),
                    runtime_state.clone(),
                    runtime_config,
                    true,
                );
            }

            if !is_fullscreen_blocked && runtime_config.lock_keys_enabled {
                if let Some((title, status_on)) = resolve_lock_action(flags, vk_code) {
                    let runtime_state = runtime_state.clone();
                    let lock_flyout_weak = lock_flyout_weak.clone();
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(lock_flyout) = lock_flyout_weak.upgrade() {
                            show_lock_flyout(
                                &lock_flyout,
                                &runtime_state,
                                &title,
                                status_on,
                                runtime_config,
                            );
                        }
                    });
                }
            }
        }
    });
}

fn setup_tray_icon(
    app_weak: slint::Weak<AppWindow>,
    settings_window_weak: slint::Weak<SettingsWindow>,
    live_flyout_weak: slint::Weak<LiveFlyoutWindow>,
    taskbar_widget_weak: slint::Weak<TaskbarWidgetWindow>,
    runtime_state: Arc<RuntimeState>,
    runtime_config: RuntimeConfig,
) -> Option<TrayResources> {
    let icon_path = std::path::PathBuf::from(
        "/mnt/c/Users/minse/.gemini/antigravity/playground/spatial-kilonova/FluentFlyout/FluentFlyoutWPF/Resources/FluentFlyout2.ico",
    );
    let icon = TrayIconImage::from_path(icon_path, Some((32, 32))).ok()?;

    let tray_menu = Menu::new();
    let settings_item = MenuItem::with_id("settings", "Open Settings", true, None);
    let dashboard_item = MenuItem::with_id("dashboard", "Open Preview Dashboard", true, None);
    let media_item = MenuItem::with_id("media", "Show Media Flyout", true, None);
    let quit_item = MenuItem::with_id("quit", "Quit", true, None);
    let separator = PredefinedMenuItem::separator();
    let _ = tray_menu.append_items(&[
        &settings_item,
        &dashboard_item,
        &media_item,
        &separator,
        &quit_item,
    ]);

    let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("FluentFlyout")
        .with_icon(icon)
        .with_menu_on_left_click(false)
        .build()
        .ok()?;

    let tray_timer = slint::Timer::default();
    tray_timer.start(
        slint::TimerMode::Repeated,
        Duration::from_millis(150),
        move || {
            while let Ok(event) = MenuEvent::receiver().try_recv() {
                match event.id().0.as_str() {
                    "settings" => {
                        if let Some(window) = settings_window_weak.upgrade() {
                            let _ = window.show();
                        }
                    }
                    "dashboard" => {
                        if let Some(app) = app_weak.upgrade() {
                            let _ = app.show();
                        }
                    }
                    "media" => {
                        refresh_live_media(
                            app_weak.clone(),
                            live_flyout_weak.clone(),
                            taskbar_widget_weak.clone(),
                            runtime_state.clone(),
                            runtime_config,
                            true,
                        );
                    }
                    "quit" => {
                        let _ = slint::quit_event_loop();
                    }
                    _ => {}
                }
            }

            while let Ok(event) = TrayIconEvent::receiver().try_recv() {
                if let TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                } = event
                {
                    if runtime_config.nicon_left_click == 1 {
                        refresh_live_media(
                            app_weak.clone(),
                            live_flyout_weak.clone(),
                            taskbar_widget_weak.clone(),
                            runtime_state.clone(),
                            runtime_config,
                            true,
                        );
                    } else if let Some(window) = settings_window_weak.upgrade() {
                        let _ = window.show();
                    }
                }
            }
        },
    );

    Some(TrayResources {
        _tray_icon: tray_icon,
        _settings_item: settings_item,
        _dashboard_item: dashboard_item,
        _media_item: media_item,
        _quit_item: quit_item,
        _tray_timer: tray_timer,
    })
}

extern "C" fn keyboard_hook_callback(vk_code: i32) {
    if let Some(sender) = KEYBOARD_EVENT_SENDER.get() {
        let _ = sender.send(vk_code);
    }
}

fn refresh_live_media(
    app_weak: slint::Weak<AppWindow>,
    live_flyout_weak: slint::Weak<LiveFlyoutWindow>,
    taskbar_widget_weak: slint::Weak<TaskbarWidgetWindow>,
    runtime_state: Arc<RuntimeState>,
    runtime_config: RuntimeConfig,
    force_present: bool,
) {
    std::thread::spawn(move || {
        let snapshot = media::load_current_media_snapshot(runtime_config.exclusive_tidal_mode)
            .ok()
            .flatten();
        let is_fullscreen_blocked =
            runtime_config.disable_if_fullscreen && is_fullscreen_app_running();
        let should_present = !is_fullscreen_blocked
            && runtime_config.media_flyout_enabled
            && snapshot.is_some()
            && (runtime_config.keep_visible_when_idle
                || force_present
                || should_present_for_snapshot(&runtime_state, snapshot.as_ref()));

        let _ = slint::invoke_from_event_loop(move || {
            if let Some(app) = app_weak.upgrade() {
                apply_live_media_snapshot_to_dashboard(&app, snapshot.as_ref());
            }

            if let Some(taskbar_widget) = taskbar_widget_weak.upgrade() {
                apply_live_media_snapshot_to_taskbar_widget(
                    &taskbar_widget,
                    snapshot.as_ref(),
                    runtime_config,
                );

                let should_show_widget = runtime_config.taskbar_widget_enabled
                    && !is_fullscreen_blocked
                    && (snapshot.is_some() || !runtime_config.taskbar_widget_hide_completely);

                if should_show_widget {
                    let _ = taskbar_widget.show();
                    apply_taskbar_widget_position(&taskbar_widget, runtime_config);
                } else {
                    let _ = taskbar_widget.hide();
                }
            }

            if let Some(live_flyout) = live_flyout_weak.upgrade() {
                apply_live_media_snapshot_to_flyout(&live_flyout, snapshot.as_ref());

                if should_present {
                    let _ = live_flyout.show();
                    apply_live_flyout_position(
                        &live_flyout,
                        runtime_config.flyout_position,
                        runtime_config.flyout_selected_monitor,
                    );
                    let epoch = bump_media_visibility_epoch(&runtime_state);
                    if !runtime_config.keep_visible_when_idle {
                        schedule_media_flyout_hide(
                            live_flyout.as_weak(),
                            runtime_state.clone(),
                            epoch,
                            runtime_config.flyout_duration_ms,
                        );
                    }
                } else if runtime_config.media_flyout_enabled
                    && runtime_config.keep_visible_when_idle
                    && snapshot.is_some()
                    && !is_fullscreen_blocked
                {
                    let _ = live_flyout.show();
                    apply_live_flyout_position(
                        &live_flyout,
                        runtime_config.flyout_position,
                        runtime_config.flyout_selected_monitor,
                    );
                    bump_media_visibility_epoch(&runtime_state);
                } else if snapshot.is_none()
                    || !runtime_config.media_flyout_enabled
                    || is_fullscreen_blocked
                {
                    bump_media_visibility_epoch(&runtime_state);
                    let _ = live_flyout.hide();
                }
            }
        });
    });
}

fn apply_live_media_snapshot_to_dashboard(
    app: &AppWindow,
    snapshot: Option<&media::MediaSnapshot>,
) {
    if let Some(snapshot) = snapshot {
        let album_art = decode_album_art(snapshot.album_art_bytes.as_deref()).unwrap_or_default();

        app.set_normalTitle(snapshot.title.clone().into());
        app.set_normalArtist(snapshot.artist.clone().into());
        app.set_normalPlayer(snapshot.player_label.clone().into());
        app.set_normalCurrentDuration(snapshot.current_duration.clone().into());
        app.set_normalMaxDuration(snapshot.max_duration.clone().into());
        app.set_normalAlbumArt(album_art.clone());
        app.set_normalHasAlbumArt(snapshot.has_album_art);
        app.set_normalSeekbarValue(snapshot.position_seconds);
        app.set_normalSeekbarMaximum(snapshot.max_seek_seconds.max(1.0));
        app.set_normalRepeatLabel(snapshot.repeat_label.clone().into());
        app.set_normalRepeatActive(snapshot.repeat_active);
        app.set_normalRepeatAvailable(snapshot.can_toggle_repeat);
        app.set_normalShuffleActive(snapshot.shuffle_active);
        app.set_normalShuffleAvailable(snapshot.can_toggle_shuffle);
        app.set_normalMediaSessionSupportsSeekbar(snapshot.supports_seekbar);
        app.set_normalPlayPauseLabel(if snapshot.is_playing { "II" } else { ">" }.into());

        app.set_compactTitle(snapshot.title.clone().into());
        app.set_compactArtist(snapshot.artist.clone().into());
        app.set_compactPlayer(snapshot.player_label.clone().into());
        app.set_compactCurrentDuration(snapshot.current_duration.clone().into());
        app.set_compactMaxDuration(snapshot.max_duration.clone().into());
        app.set_compactAlbumArt(album_art);
        app.set_compactHasAlbumArt(snapshot.has_album_art);
        app.set_compactSeekbarValue(snapshot.position_seconds);
        app.set_compactSeekbarMaximum(snapshot.max_seek_seconds.max(1.0));
        app.set_compactRepeatLabel(snapshot.repeat_label.clone().into());
        app.set_compactRepeatActive(snapshot.repeat_active);
        app.set_compactRepeatAvailable(snapshot.can_toggle_repeat);
        app.set_compactShuffleActive(snapshot.shuffle_active);
        app.set_compactShuffleAvailable(snapshot.can_toggle_shuffle);
        app.set_compactPlayPauseLabel(if snapshot.is_playing { "II" } else { ">" }.into());
        return;
    }

    app.set_normalTitle("No media playing".into());
    app.set_normalArtist(String::new().into());
    app.set_normalPlayer("Idle".into());
    app.set_normalCurrentDuration("-:--".into());
    app.set_normalMaxDuration("-:--".into());
    app.set_normalAlbumArt(Image::default());
    app.set_normalHasAlbumArt(false);
    app.set_normalSeekbarValue(0.0);
    app.set_normalSeekbarMaximum(1.0);
    app.set_normalRepeatLabel("R".into());
    app.set_normalRepeatActive(false);
    app.set_normalRepeatAvailable(false);
    app.set_normalShuffleActive(false);
    app.set_normalShuffleAvailable(false);
    app.set_normalMediaSessionSupportsSeekbar(false);
    app.set_normalPlayPauseLabel(">".into());

    app.set_compactTitle("No media playing".into());
    app.set_compactArtist(String::new().into());
    app.set_compactPlayer("Idle".into());
    app.set_compactCurrentDuration("-:--".into());
    app.set_compactMaxDuration("-:--".into());
    app.set_compactAlbumArt(Image::default());
    app.set_compactHasAlbumArt(false);
    app.set_compactSeekbarValue(0.0);
    app.set_compactSeekbarMaximum(1.0);
    app.set_compactRepeatLabel("R".into());
    app.set_compactRepeatActive(false);
    app.set_compactRepeatAvailable(false);
    app.set_compactShuffleActive(false);
    app.set_compactShuffleAvailable(false);
    app.set_compactPlayPauseLabel(">".into());
}

fn apply_live_media_snapshot_to_flyout(
    live_flyout: &LiveFlyoutWindow,
    snapshot: Option<&media::MediaSnapshot>,
) {
    if let Some(snapshot) = snapshot {
        live_flyout.set_titleText(snapshot.title.clone().into());
        live_flyout.set_artistText(snapshot.artist.clone().into());
        live_flyout.set_playerText(snapshot.player_label.clone().into());
        live_flyout.set_currentDurationText(snapshot.current_duration.clone().into());
        live_flyout.set_maxDurationText(snapshot.max_duration.clone().into());
        live_flyout.set_albumArt(
            decode_album_art(snapshot.album_art_bytes.as_deref()).unwrap_or_default(),
        );
        live_flyout.set_hasAlbumArt(snapshot.has_album_art);
        live_flyout.set_seekbarValue(snapshot.position_seconds);
        live_flyout.set_seekbarMaximum(snapshot.max_seek_seconds.max(1.0));
        live_flyout.set_repeatLabel(snapshot.repeat_label.clone().into());
        live_flyout.set_repeatActive(snapshot.repeat_active);
        live_flyout.set_repeatAvailable(snapshot.can_toggle_repeat);
        live_flyout.set_shuffleActive(snapshot.shuffle_active);
        live_flyout.set_shuffleAvailable(snapshot.can_toggle_shuffle);
        live_flyout.set_mediaSessionSupportsSeekbar(snapshot.supports_seekbar);
        live_flyout.set_playPauseLabel(if snapshot.is_playing { "II" } else { ">" }.into());
        return;
    }

    live_flyout.set_titleText("No media playing".into());
    live_flyout.set_artistText(String::new().into());
    live_flyout.set_playerText("Idle".into());
    live_flyout.set_currentDurationText("-:--".into());
    live_flyout.set_maxDurationText("-:--".into());
    live_flyout.set_albumArt(Image::default());
    live_flyout.set_hasAlbumArt(false);
    live_flyout.set_seekbarValue(0.0);
    live_flyout.set_seekbarMaximum(1.0);
    live_flyout.set_repeatLabel("R".into());
    live_flyout.set_repeatActive(false);
    live_flyout.set_repeatAvailable(false);
    live_flyout.set_shuffleActive(false);
    live_flyout.set_shuffleAvailable(false);
    live_flyout.set_mediaSessionSupportsSeekbar(false);
    live_flyout.set_playPauseLabel(">".into());
}

fn apply_live_media_snapshot_to_taskbar_widget(
    taskbar_widget: &TaskbarWidgetWindow,
    snapshot: Option<&media::MediaSnapshot>,
    runtime_config: RuntimeConfig,
) {
    taskbar_widget.set_controlsEnabled(runtime_config.taskbar_widget_controls_enabled);
    taskbar_widget.set_controlsPosition(runtime_config.taskbar_widget_controls_position);
    taskbar_widget.set_backgroundBlur(runtime_config.taskbar_widget_background_blur);

    if let Some(snapshot) = snapshot {
        taskbar_widget.set_titleText(snapshot.title.clone().into());
        taskbar_widget.set_artistText(snapshot.artist.clone().into());
        taskbar_widget.set_playerText(snapshot.player_label.clone().into());
        taskbar_widget.set_albumArt(
            decode_album_art(snapshot.album_art_bytes.as_deref()).unwrap_or_default(),
        );
        taskbar_widget.set_hasAlbumArt(snapshot.has_album_art);
        taskbar_widget.set_hasMedia(true);
        taskbar_widget.set_isPlaying(snapshot.is_playing);
        taskbar_widget.set_playPauseLabel(if snapshot.is_playing { "II" } else { ">" }.into());
        return;
    }

    taskbar_widget.set_titleText("No media playing".into());
    taskbar_widget.set_artistText(String::new().into());
    taskbar_widget.set_playerText("Idle".into());
    taskbar_widget.set_albumArt(Image::default());
    taskbar_widget.set_hasAlbumArt(false);
    taskbar_widget.set_hasMedia(false);
    taskbar_widget.set_isPlaying(false);
    taskbar_widget.set_playPauseLabel(">".into());
}

fn show_lock_flyout(
    lock_flyout: &LockFlyoutWindow,
    runtime_state: &Arc<RuntimeState>,
    title: &str,
    status_on: bool,
    runtime_config: RuntimeConfig,
) {
    lock_flyout.set_titleText(title.into());
    lock_flyout.set_statusOn(status_on);
    lock_flyout.set_boldUi(runtime_config.lock_keys_bold_ui);
    let _ = lock_flyout.show();
    apply_lock_flyout_position(
        lock_flyout,
        runtime_config.lock_keys_monitor_preference,
        runtime_config.flyout_selected_monitor,
    );

    let epoch = bump_lock_visibility_epoch(runtime_state);
    schedule_lock_flyout_hide(
        lock_flyout.as_weak(),
        runtime_state.clone(),
        epoch,
        runtime_config.lock_keys_duration_ms,
    );
}

fn apply_live_flyout_position(
    live_flyout: &LiveFlyoutWindow,
    position: i32,
    selected_monitor: i32,
) {
    let monitor = monitor::get_selected_monitor(selected_monitor);
    let size = live_flyout.window().size();
    let width = size.width as i32;
    let height = size.height as i32;

    let x = match position {
        0 | 3 => monitor.left + 16,
        1 | 4 => monitor.left + (monitor.width - width) / 2,
        2 | 5 => monitor.left + monitor.width - width - 16,
        _ => monitor.left + 16,
    };

    let y = match position {
        0 | 2 => monitor.top + monitor.height - height - 16,
        1 => monitor.top + monitor.height - height - 80,
        3..=5 => monitor.top + 16,
        _ => monitor.top + monitor.height - height - 16,
    };

    live_flyout.window().set_position(PhysicalPosition::new(x, y));
}

fn apply_taskbar_widget_position(taskbar_widget: &TaskbarWidgetWindow, runtime_config: RuntimeConfig) {
    let size = taskbar_widget.window().size();
    let width = size.width as i32;
    let height = size.height as i32;

    if let Some(hwnd) = slint_window_hwnd(taskbar_widget.window()) {
        if taskbar_native::place_widget_in_taskbar(
            hwnd,
            runtime_config.taskbar_widget_selected_monitor,
            runtime_config.taskbar_widget_position,
            runtime_config.taskbar_widget_padding,
            runtime_config.taskbar_widget_manual_padding,
            width,
            height,
        ) {
            return;
        }
    }

    let monitor = monitor::get_selected_monitor(runtime_config.taskbar_widget_selected_monitor);
    let x = match runtime_config.taskbar_widget_position {
        0 => monitor.left + 20 + runtime_config.taskbar_widget_manual_padding,
        1 => monitor.left + (monitor.width - width) / 2,
        2 => monitor.left + monitor.width - width - 20 + runtime_config.taskbar_widget_manual_padding,
        _ => monitor.left + 20,
    };
    let y = monitor.top + monitor.height - height;
    taskbar_widget.window().set_position(PhysicalPosition::new(x, y));
}

fn slint_window_hwnd(window: &slint::Window) -> Option<HWND> {
    let handle = window.window_handle();
    let raw = handle.window_handle().ok()?;
    match raw.as_raw() {
        RawWindowHandle::Win32(handle) => Some(HWND(handle.hwnd.get() as *mut _)),
        _ => None,
    }
}

fn apply_lock_flyout_position(
    lock_flyout: &LockFlyoutWindow,
    lock_monitor_preference: i32,
    selected_monitor: i32,
) {
    let monitor = monitor::get_lock_target_monitor(lock_monitor_preference, selected_monitor);
    let size = lock_flyout.window().size();
    let width = size.width as i32;
    let height = size.height as i32;
    let x = monitor.left + (monitor.width - width) / 2;
    let y = monitor.top + monitor.height - height - 16;
    lock_flyout.window().set_position(PhysicalPosition::new(x, y));
}

fn resolve_lock_action(flags: u32, vk_code: i32) -> Option<(String, bool)> {
    if (flags & INPUT_ACTION_LOCK_CAPS) != 0 {
        let is_on = current_toggle_state(vk_code);
        return Some((format!("Caps Lock {}", if is_on { "On" } else { "Off" }), is_on));
    }

    if (flags & INPUT_ACTION_LOCK_NUM) != 0 {
        let is_on = current_toggle_state(vk_code);
        return Some((format!("Num Lock {}", if is_on { "On" } else { "Off" }), is_on));
    }

    if (flags & INPUT_ACTION_LOCK_SCROLL) != 0 {
        let is_on = current_toggle_state(vk_code);
        return Some((format!("Scroll Lock {}", if is_on { "On" } else { "Off" }), is_on));
    }

    if (flags & INPUT_ACTION_LOCK_INSERT) != 0 {
        return Some((String::from("Insert Pressed"), true));
    }

    None
}

fn current_toggle_state(vk_code: i32) -> bool {
    unsafe { GetKeyState(vk_code) & 1 != 0 }
}

fn should_present_for_snapshot(
    runtime_state: &Arc<RuntimeState>,
    snapshot: Option<&media::MediaSnapshot>,
) -> bool {
    let next_key = snapshot.map(|value| value.session_key.clone());
    let Ok(mut last_key) = runtime_state.last_session_key.lock() else {
        return snapshot.is_some();
    };

    if *last_key == next_key {
        return false;
    }

    *last_key = next_key;
    snapshot.is_some()
}

fn should_debounce_media_trigger(runtime_state: &Arc<RuntimeState>) -> bool {
    let now = current_time_ms();
    let last = runtime_state.last_media_input_ms.load(Ordering::SeqCst);
    if now.saturating_sub(last) < 500 {
        return false;
    }

    runtime_state.last_media_input_ms.store(now, Ordering::SeqCst);
    true
}

fn bump_media_visibility_epoch(runtime_state: &Arc<RuntimeState>) -> u64 {
    runtime_state
        .media_visibility_epoch
        .fetch_add(1, Ordering::SeqCst)
        .saturating_add(1)
}

fn bump_lock_visibility_epoch(runtime_state: &Arc<RuntimeState>) -> u64 {
    runtime_state
        .lock_visibility_epoch
        .fetch_add(1, Ordering::SeqCst)
        .saturating_add(1)
}

fn schedule_media_flyout_hide(
    live_flyout_weak: slint::Weak<LiveFlyoutWindow>,
    runtime_state: Arc<RuntimeState>,
    epoch: u64,
    delay_ms: u64,
) {
    let delay = Duration::from_millis(delay_ms.max(250));

    std::thread::spawn(move || {
        std::thread::sleep(delay);

        let _ = slint::invoke_from_event_loop(move || {
            if runtime_state.media_visibility_epoch.load(Ordering::SeqCst) != epoch {
                return;
            }

            if let Some(flyout) = live_flyout_weak.upgrade() {
                let _ = flyout.hide();
            }
        });
    });
}

fn schedule_lock_flyout_hide(
    lock_flyout_weak: slint::Weak<LockFlyoutWindow>,
    runtime_state: Arc<RuntimeState>,
    epoch: u64,
    delay_ms: u64,
) {
    let delay = Duration::from_millis(delay_ms.max(250));

    std::thread::spawn(move || {
        std::thread::sleep(delay);

        let _ = slint::invoke_from_event_loop(move || {
            if runtime_state.lock_visibility_epoch.load(Ordering::SeqCst) != epoch {
                return;
            }

            if let Some(lock_flyout) = lock_flyout_weak.upgrade() {
                let _ = lock_flyout.hide();
            }
        });
    });
}

fn current_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn decode_album_art(bytes: Option<&[u8]>) -> Option<Image> {
    let rgba = ImageReader::new(std::io::Cursor::new(bytes?))
        .with_guessed_format()
        .ok()?
        .decode()
        .ok()?
        .to_rgba8();

    let (width, height) = rgba.dimensions();
    let mut pixels = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::new(width, height);

    for (target, source) in pixels.make_mut_slice().iter_mut().zip(rgba.pixels()) {
        *target = slint::Rgba8Pixel {
            r: source[0],
            g: source[1],
            b: source[2],
            a: source[3],
        };
    }

    Some(Image::from_rgba8(pixels))
}

fn animation_tail_ms(speed: i32, easing_style: i32) -> u64 {
    let base: u64 = match speed {
        0 => 0,
        1 => 150,
        2 => 300,
        3 => 450,
        4 => 600,
        _ => 900,
    };

    if easing_style == 0 {
        base
    } else {
        base.saturating_add(30)
    }
}

fn run_media_action(exclusive_tidal_mode: bool, action: fn(bool) -> Result<bool, String>) {
    std::thread::spawn(move || {
        let _ = action(exclusive_tidal_mode);
    });
}

fn run_seek_action(exclusive_tidal_mode: bool, seconds: f32) {
    std::thread::spawn(move || {
        let _ = media::seek_to_seconds(exclusive_tidal_mode, seconds);
    });
}

struct TrayResources {
    _tray_icon: TrayIcon,
    _settings_item: MenuItem,
    _dashboard_item: MenuItem,
    _media_item: MenuItem,
    _quit_item: MenuItem,
    _tray_timer: slint::Timer,
}





