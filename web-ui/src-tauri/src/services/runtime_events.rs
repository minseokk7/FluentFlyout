use std::{
    sync::{mpsc, OnceLock},
    thread,
    time::Duration,
};

use crate::services::{audio_meter, flyout_window, media_session, settings::SettingsService, taskbar_widget};

#[derive(Debug, Clone, Copy)]
enum RuntimeEvent {
    MediaKey,
    LockKey { name: &'static str, enabled: bool },
}

static STARTED: OnceLock<()> = OnceLock::new();

pub fn start<R: tauri::Runtime>(app: tauri::AppHandle<R>, settings: SettingsService) {
    if STARTED.set(()).is_err() {
        return;
    }

    let (tx, rx) = mpsc::channel::<RuntimeEvent>();
    let (widget_tx, widget_rx) = mpsc::channel::<taskbar_widget::TaskbarWidgetAction>();
    taskbar_widget::set_click_sender(widget_tx);
    start_keyboard_hook(tx);
    start_media_poll(app.clone(), settings.clone());

    {
        let app = app.clone();
        let settings = settings.clone();
        thread::spawn(move || {
            while let Ok(action) = widget_rx.recv() {
                let app = app.clone();
                let settings = settings.clone();
                tauri::async_runtime::spawn(async move {
                    let Ok(current) = settings.load().await else { return; };
                    match action {
                        taskbar_widget::TaskbarWidgetAction::ShowFlyout => {
                            if current.taskbar_widget_clickable {
                                let _ = flyout_window::show_media_flyout(&app, &current);
                            }
                        }
                        taskbar_widget::TaskbarWidgetAction::Previous => {
                            let _ = media_session::control("previous".to_string()).await;
                        }
                        taskbar_widget::TaskbarWidgetAction::PlayPause => {
                            let _ = media_session::control("playPause".to_string()).await;
                        }
                        taskbar_widget::TaskbarWidgetAction::Next => {
                            let _ = media_session::control("next".to_string()).await;
                        }
                    }
                });
            }
        });
    }

    thread::spawn(move || {
        while let Ok(event) = rx.recv() {
            let app = app.clone();
            let settings = settings.clone();
            tauri::async_runtime::spawn(async move {
                let Ok(current) = settings.load().await else { return; };
                match event {
                    RuntimeEvent::MediaKey => {
                        let _ = flyout_window::show_media_flyout(&app, &current);
                    }
                    RuntimeEvent::LockKey { name, enabled } => {
                        let _ = flyout_window::show_lock_keys_flyout(&app, &current, name, enabled);
                    }
                }
            });
        }
    });
}

fn start_media_poll<R: tauri::Runtime>(app: tauri::AppHandle<R>, settings: SettingsService) {
    thread::spawn(move || {
        let mut last_title = String::new();
        loop {
            thread::sleep(Duration::from_millis(1500));
            let Ok(current) = tauri::async_runtime::block_on(settings.load()) else { continue; };

            let Ok(media) = tauri::async_runtime::block_on(media_session::get_media_session()) else { continue; };
            taskbar_widget::update_media(&media);

            if current.taskbar_widget_enabled {
                let _ = taskbar_widget::apply_settings_on_main_thread(&app, current.clone());
            }

            if current.taskbar_visualizer_enabled {
                if let Ok(peak) = audio_meter::output_peak(&current) {
                    taskbar_widget::update_audio_peak(peak);
                }
            }

            let title_key = format!("{}|{}", media.title, media.artist);
            let has_media = !media.title.trim().is_empty() && media.title != "No media playing";
            if has_media && !last_title.is_empty() && title_key != last_title {
                let _ = flyout_window::show_next_up_flyout(&app, &current, &media);
            }
            if has_media {
                last_title = title_key;
            }
        }
    });
}

#[cfg(windows)]
fn start_keyboard_hook(tx: mpsc::Sender<RuntimeEvent>) {
    thread::spawn(move || unsafe {
        use windows::Win32::UI::WindowsAndMessaging::{
            DispatchMessageW, GetMessageW, SetWindowsHookExW, TranslateMessage, HHOOK, MSG, WH_KEYBOARD_LL,
        };

        static KEY_TX: OnceLock<mpsc::Sender<RuntimeEvent>> = OnceLock::new();
        let _ = KEY_TX.set(tx);

        unsafe extern "system" fn hook_proc(
            code: i32,
            wparam: windows::Win32::Foundation::WPARAM,
            lparam: windows::Win32::Foundation::LPARAM,
        ) -> windows::Win32::Foundation::LRESULT {
            use windows::Win32::UI::Input::KeyboardAndMouse::{
                VK_CAPITAL, VK_INSERT, VK_MEDIA_NEXT_TRACK, VK_MEDIA_PLAY_PAUSE,
                VK_MEDIA_PREV_TRACK, VK_NUMLOCK, VK_SCROLL, VK_VOLUME_DOWN, VK_VOLUME_MUTE, VK_VOLUME_UP,
            };
            use windows::Win32::UI::WindowsAndMessaging::{CallNextHookEx, KBDLLHOOKSTRUCT, WM_KEYUP, WM_SYSKEYUP};

            if code >= 0 && (wparam.0 as u32 == WM_KEYUP || wparam.0 as u32 == WM_SYSKEYUP) {
                let info = unsafe { *(lparam.0 as *const KBDLLHOOKSTRUCT) };
                let event = match info.vkCode {
                    code if code == VK_MEDIA_PLAY_PAUSE.0 as u32
                        || code == VK_MEDIA_NEXT_TRACK.0 as u32
                        || code == VK_MEDIA_PREV_TRACK.0 as u32
                        || code == VK_VOLUME_UP.0 as u32
                        || code == VK_VOLUME_DOWN.0 as u32
                        || code == VK_VOLUME_MUTE.0 as u32 => Some(RuntimeEvent::MediaKey),
                    code if code == VK_CAPITAL.0 as u32 => Some(RuntimeEvent::LockKey { name: "Caps Lock", enabled: key_enabled(VK_CAPITAL.0 as i32) }),
                    code if code == VK_NUMLOCK.0 as u32 => Some(RuntimeEvent::LockKey { name: "Num Lock", enabled: key_enabled(VK_NUMLOCK.0 as i32) }),
                    code if code == VK_SCROLL.0 as u32 => Some(RuntimeEvent::LockKey { name: "Scroll Lock", enabled: key_enabled(VK_SCROLL.0 as i32) }),
                    code if code == VK_INSERT.0 as u32 => Some(RuntimeEvent::LockKey { name: "Insert", enabled: key_enabled(VK_INSERT.0 as i32) }),
                    _ => None,
                };
                if let Some(event) = event {
                    if let Some(tx) = KEY_TX.get() {
                        let _ = tx.send(event);
                    }
                }
            }
            unsafe { CallNextHookEx(None, code, wparam, lparam) }
        }

        fn key_enabled(vk: i32) -> bool {
            unsafe { (windows::Win32::UI::Input::KeyboardAndMouse::GetKeyState(vk) & 1) != 0 }
        }

        let hook: HHOOK = SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_proc), None, 0).unwrap_or_default();
        if hook.0.is_null() {
            return;
        }
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    });
}

#[cfg(not(windows))]
fn start_keyboard_hook(_tx: mpsc::Sender<RuntimeEvent>) {}
