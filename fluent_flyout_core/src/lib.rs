// Copyright © 2024-2026 The FluentFlyout Authors
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::{OnceLock, Mutex, mpsc};
use std::ffi::CString;
use std::os::raw::c_char;
use std::thread;
use std::fs::OpenOptions;
use std::io::Write;
use windows::Media::Control::*;
use windows::Win32::UI::WindowsAndMessaging::{CallNextHookEx, SetWindowsHookExW, UnhookWindowsHookEx, HHOOK, WH_KEYBOARD_LL, KBDLLHOOKSTRUCT};
use windows::Win32::Foundation::{WPARAM, LPARAM, LRESULT, HINSTANCE, HMODULE};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Diagnostics::Debug::OutputDebugStringW;
use windows::Win32::UI::Shell::{SHQueryUserNotificationState, QUNS_RUNNING_D3D_FULL_SCREEN};
use windows::core::w;
use windows::Media::MediaPlaybackAutoRepeatMode;

use rustfft::{FftPlanner, num_complex::Complex};

// --- Error Handling & Logging ---

static LAST_ERROR: Mutex<String> = Mutex::new(String::new());

fn set_last_error(msg: &str) {
    let mut err = LAST_ERROR.lock().unwrap();
    *err = msg.to_string();
    log_debug(msg);
}

#[unsafe(no_mangle)]
pub extern "C" fn GetLastRustError() -> *mut c_char {
    let err = LAST_ERROR.lock().unwrap();
    to_c_string(&err)
}

fn log_debug(msg: &str) {
    let wide: Vec<u16> = msg.encode_utf16().chain(std::iter::once(0)).collect();
    unsafe { OutputDebugStringW(windows::core::PCWSTR(wide.as_ptr())); }
    
    // Log to file as well for robustness
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open("fluent_flyout_debug.log") {
        let _ = writeln!(file, "{}", msg);
    }
}

// --- Native Structs (Matching C#) ---

#[repr(C)]
pub struct MediaPlayerProcessMatchNative {
    pub path: *mut c_char,
    pub window_title: *mut c_char,
}

#[repr(C)]
pub struct RustByteBufferNative {
    pub ptr: *mut u8,
    pub len: usize,
}

#[repr(C)]
pub struct MediaSessionSnapshotNative {
    pub session_id: *mut c_char,
    pub title: *mut c_char,
    pub artist: *mut c_char,
    pub playback_status: i32,
    pub is_play_enabled: u8,
    pub is_pause_enabled: u8,
    pub is_previous_enabled: u8,
    pub is_next_enabled: u8,
    pub is_repeat_enabled: u8,
    pub is_shuffle_enabled: u8,
    pub repeat_mode: i32,
    pub is_shuffle_active: u8,
    pub position_ticks: i64,
    pub end_time_ticks: i64,
    pub max_seek_time_ticks: i64,
    pub last_updated_ticks: i64,
}

// --- Globals & Helpers ---

#[derive(Clone, Copy)]
struct SendHandle<T>(T);
unsafe impl<T> Send for SendHandle<T> {}
unsafe impl<T> Sync for SendHandle<T> {}

static MANAGER: OnceLock<GlobalSystemMediaTransportControlsSessionManager> = OnceLock::new();
static FFT_PLANNER: OnceLock<Mutex<FftPlanner<f32>>> = OnceLock::new();

static KEYBOARD_HOOK: Mutex<Option<SendHandle<HHOOK>>> = Mutex::new(None);
static CALLBACK: OnceLock<extern "C" fn(i32)> = OnceLock::new();
static EVENT_SENDER: OnceLock<mpsc::Sender<i32>> = OnceLock::new();

struct InputConfig {
    media_flyout_enabled: bool,
    volume_keys_excluded: bool,
    lock_keys_enabled: bool,
    lock_insert_enabled: bool,
}

static INPUT_CONFIG: Mutex<InputConfig> = Mutex::new(InputConfig {
    media_flyout_enabled: true,
    volume_keys_excluded: false,
    lock_keys_enabled: true,
    lock_insert_enabled: true,
});

fn get_manager() -> Option<&'static GlobalSystemMediaTransportControlsSessionManager> {
    MANAGER.get()
}

fn get_fft_planner() -> &'static Mutex<FftPlanner<f32>> {
    FFT_PLANNER.get_or_init(|| Mutex::new(FftPlanner::new()))
}

// --- String Helpers ---

fn to_c_string(s: &str) -> *mut c_char {
    CString::new(s).unwrap().into_raw()
}

#[unsafe(no_mangle)]
pub extern "C" fn free_string(s: *mut c_char) {
    if s.is_null() { return; }
    unsafe {
        let _ = CString::from_raw(s);
    }
}

// --- Media Methods ---

#[unsafe(no_mangle)]
pub extern "C" fn get_media_title(_exclusive: bool) -> *mut c_char {
    if let Some(manager) = get_manager() {
        if let Ok(session) = manager.GetCurrentSession() {
            if let Ok(props_op) = session.TryGetMediaPropertiesAsync() {
                if let Ok(props) = props_op.get() {
                    if let Ok(title) = props.Title() {
                        return to_c_string(&title.to_string());
                    }
                }
            }
        }
    }
    std::ptr::null_mut()
}

#[unsafe(no_mangle)]
pub extern "C" fn get_media_artist(_exclusive: bool) -> *mut c_char {
    if let Some(manager) = get_manager() {
        if let Ok(session) = manager.GetCurrentSession() {
            if let Ok(props_op) = session.TryGetMediaPropertiesAsync() {
                if let Ok(props) = props_op.get() {
                    if let Ok(artist) = props.Artist() {
                        return to_c_string(&artist.to_string());
                    }
                }
            }
        }
    }
    std::ptr::null_mut()
}

#[unsafe(no_mangle)]
pub extern "C" fn media_play_pause(_exclusive: bool) -> bool {
    if let Some(manager) = get_manager() {
        if let Ok(session) = manager.GetCurrentSession() {
            return session.TryTogglePlayPauseAsync().is_ok();
        }
    }
    false
}

#[unsafe(no_mangle)]
pub extern "C" fn media_next(_exclusive: bool) -> bool {
    if let Some(manager) = get_manager() {
        if let Ok(session) = manager.GetCurrentSession() {
            return session.TrySkipNextAsync().is_ok();
        }
    }
    false
}

#[unsafe(no_mangle)]
pub extern "C" fn media_previous(_exclusive: bool) -> bool {
    if let Some(manager) = get_manager() {
        if let Ok(session) = manager.GetCurrentSession() {
            return session.TrySkipPreviousAsync().is_ok();
        }
    }
    false
}

#[unsafe(no_mangle)]
pub extern "C" fn media_play(_exclusive: bool) -> bool {
    if let Some(manager) = get_manager() {
        if let Ok(session) = manager.GetCurrentSession() {
            return session.TryPlayAsync().is_ok();
        }
    }
    false
}

#[unsafe(no_mangle)]
pub extern "C" fn media_pause(_exclusive: bool) -> bool {
    if let Some(manager) = get_manager() {
        if let Ok(session) = manager.GetCurrentSession() {
            return session.TryPauseAsync().is_ok();
        }
    }
    false
}

#[unsafe(no_mangle)]
pub extern "C" fn media_cycle_repeat_mode(_exclusive: bool) -> bool {
    if let Some(manager) = get_manager() {
        if let Ok(session) = manager.GetCurrentSession() {
            if let Ok(info) = session.GetPlaybackInfo() {
                if let Ok(mode_ref) = info.AutoRepeatMode() {
                    if let Ok(mode) = mode_ref.Value() {
                        let next_mode = match mode {
                            MediaPlaybackAutoRepeatMode::None => MediaPlaybackAutoRepeatMode::List,
                            MediaPlaybackAutoRepeatMode::List => MediaPlaybackAutoRepeatMode::Track,
                            MediaPlaybackAutoRepeatMode::Track => MediaPlaybackAutoRepeatMode::None,
                            _ => MediaPlaybackAutoRepeatMode::None,
                        };
                        return session.TryChangeAutoRepeatModeAsync(next_mode).is_ok();
                    }
                }
            }
        }
    }
    false
}

#[unsafe(no_mangle)]
pub extern "C" fn media_toggle_shuffle(_exclusive: bool) -> bool {
    if let Some(manager) = get_manager() {
        if let Ok(session) = manager.GetCurrentSession() {
            if let Ok(info) = session.GetPlaybackInfo() {
                if let Ok(active_ref) = info.IsShuffleActive() {
                    if let Ok(active) = active_ref.Value() {
                        return session.TryChangeShuffleActiveAsync(!active).is_ok();
                    }
                }
            }
        }
    }
    false
}

#[unsafe(no_mangle)]
pub extern "C" fn media_change_playback_position(_exclusive: bool, ticks: i64) -> bool {
    if let Some(manager) = get_manager() {
        if let Ok(session) = manager.GetCurrentSession() {
            return session.TryChangePlaybackPositionAsync(ticks).is_ok();
        }
    }
    false
}

#[unsafe(no_mangle)]
pub extern "C" fn pause_other_media_sessions(_exclusive: bool) -> u32 {
    let mut count = 0;
    if let Some(manager) = get_manager() {
        if let Ok(sessions) = manager.GetSessions() {
            if let Ok(current) = manager.GetCurrentSession() {
                let current_id = current.SourceAppUserModelId().unwrap_or_default();
                for session in sessions {
                    let id = session.SourceAppUserModelId().unwrap_or_default();
                    if id != current_id {
                        if session.TryPauseAsync().is_ok() {
                            count += 1;
                        }
                    }
                }
            }
        }
    }
    count
}

#[unsafe(no_mangle)]
pub extern "C" fn get_media_session_snapshot(_exclusive: bool) -> MediaSessionSnapshotNative {
    if let Some(manager) = get_manager() {
        if let Ok(session) = manager.GetCurrentSession() {
            let session_id = session.SourceAppUserModelId().unwrap_or_default().to_string();
            let mut props_title = String::new();
            let mut props_artist = String::new();
            
            if let Ok(props_op) = session.TryGetMediaPropertiesAsync() {
                if let Ok(props) = props_op.get() {
                    props_title = props.Title().unwrap_or_default().to_string();
                    props_artist = props.Artist().unwrap_or_default().to_string();
                }
            }

            let mut status = 0;
            let mut is_play_enabled = 0;
            let mut is_pause_enabled = 0;
            let mut is_prev_enabled = 0;
            let mut is_next_enabled = 0;
            let mut is_repeat_enabled = 0;
            let mut is_shuffle_enabled = 0;
            let mut repeat_mode = 0;
            let mut is_shuffle_active = 0;

            if let Ok(info) = session.GetPlaybackInfo() {
                status = info.PlaybackStatus().unwrap_or(GlobalSystemMediaTransportControlsSessionPlaybackStatus::Closed).0;
                if let Ok(controls) = info.Controls() {
                    is_play_enabled = if controls.IsPlayEnabled().unwrap_or_default() { 1 } else { 0 };
                    is_pause_enabled = if controls.IsPauseEnabled().unwrap_or_default() { 1 } else { 0 };
                    is_prev_enabled = if controls.IsPreviousEnabled().unwrap_or_default() { 1 } else { 0 };
                    is_next_enabled = if controls.IsNextEnabled().unwrap_or_default() { 1 } else { 0 };
                    is_repeat_enabled = if controls.IsRepeatEnabled().unwrap_or_default() { 1 } else { 0 };
                    is_shuffle_enabled = if controls.IsShuffleEnabled().unwrap_or_default() { 1 } else { 0 };
                }
                repeat_mode = info.AutoRepeatMode().map(|v| v.Value().map(|m| m.0).unwrap_or_default()).unwrap_or_default();
                is_shuffle_active = if info.IsShuffleActive().map(|v| v.Value().unwrap_or_default()).unwrap_or_default() { 1 } else { 0 };
            }

            let mut pos = 0;
            let mut end = 0;
            let mut max_seek = 0;
            let mut updated = 0;

            if let Ok(timeline) = session.GetTimelineProperties() {
                pos = timeline.Position().unwrap_or_default().Duration;
                end = timeline.EndTime().unwrap_or_default().Duration;
                max_seek = timeline.MaxSeekTime().unwrap_or_default().Duration;
                updated = timeline.LastUpdatedTime().unwrap_or_default().UniversalTime;
            }

            return MediaSessionSnapshotNative {
                session_id: to_c_string(&session_id),
                title: to_c_string(&props_title),
                artist: to_c_string(&props_artist),
                playback_status: status,
                is_play_enabled,
                is_pause_enabled,
                is_previous_enabled: is_prev_enabled,
                is_next_enabled,
                is_repeat_enabled,
                is_shuffle_enabled,
                repeat_mode: repeat_mode,
                is_shuffle_active,
                position_ticks: pos,
                end_time_ticks: end,
                max_seek_time_ticks: max_seek,
                last_updated_ticks: updated,
            };
        }
    }

    MediaSessionSnapshotNative {
        session_id: std::ptr::null_mut(),
        title: std::ptr::null_mut(),
        artist: std::ptr::null_mut(),
        playback_status: 0,
        is_play_enabled: 0,
        is_pause_enabled: 0,
        is_previous_enabled: 0,
        is_next_enabled: 0,
        is_repeat_enabled: 0,
        is_shuffle_enabled: 0,
        repeat_mode: 0,
        is_shuffle_active: 0,
        position_ticks: 0,
        end_time_ticks: 0,
        max_seek_time_ticks: 0,
        last_updated_ticks: 0,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn free_media_session_snapshot(s: MediaSessionSnapshotNative) {
    free_string(s.session_id);
    free_string(s.title);
    free_string(s.artist);
}

#[unsafe(no_mangle)]
pub extern "C" fn get_media_session_thumbnail(_exclusive: bool) -> RustByteBufferNative {
    if let Some(manager) = get_manager() {
        if let Ok(session) = manager.GetCurrentSession() {
            if let Ok(props_op) = session.TryGetMediaPropertiesAsync() {
                if let Ok(props) = props_op.get() {
                    if let Ok(thumbnail) = props.Thumbnail() {
                        if let Ok(stream_op) = thumbnail.OpenReadAsync() {
                            if let Ok(stream) = stream_op.get() {
                                let size = stream.Size().unwrap() as usize;
                                let mut buffer = vec![0u8; size];
                                let reader = windows::Storage::Streams::DataReader::CreateDataReader(&stream).unwrap();
                                reader.LoadAsync(size as u32).unwrap().get().unwrap();
                                reader.ReadBytes(&mut buffer).unwrap();
                                
                                let len = buffer.len();
                                let ptr = buffer.as_mut_ptr();
                                std::mem::forget(buffer);
                                return RustByteBufferNative { ptr, len };
                            }
                        }
                    }
                }
            }
        }
    }
    RustByteBufferNative { ptr: std::ptr::null_mut(), len: 0 }
}

#[unsafe(no_mangle)]
pub extern "C" fn free_rust_byte_buffer(b: RustByteBufferNative) {
    if !b.ptr.is_null() {
        unsafe {
            let _ = Vec::from_raw_parts(b.ptr, b.len, b.len);
        }
    }
}

// --- Process & Icon Methods ---

#[unsafe(no_mangle)]
pub extern "C" fn resolve_media_player_process(_id: *const c_char) -> MediaPlayerProcessMatchNative {
    MediaPlayerProcessMatchNative { path: std::ptr::null_mut(), window_title: std::ptr::null_mut() }
}

#[unsafe(no_mangle)]
pub extern "C" fn extract_file_icon_handle(_path: *const c_char) -> isize { 0 }
#[unsafe(no_mangle)]
pub extern "C" fn destroy_icon_handle(_handle: isize) {}

// --- Config & Input Methods ---

#[unsafe(no_mangle)]
pub extern "C" fn update_input_config(m: bool, v: bool, l: bool, li: bool) {
    let mut config = INPUT_CONFIG.lock().unwrap();
    config.media_flyout_enabled = m;
    config.volume_keys_excluded = v;
    config.lock_keys_enabled = l;
    config.lock_insert_enabled = li;
}

#[unsafe(no_mangle)]
pub extern "C" fn get_keyboard_action_flags(vk_code: i32) -> u32 {
    let config = INPUT_CONFIG.lock().unwrap();
    let mut flags = 0u32;
    
    let vk = vk_code as u16;
    let is_media = (0xB0..=0xB3).contains(&vk);
    let is_volume = (0xAD..=0xAF).contains(&vk);
    
    if config.media_flyout_enabled && (is_media || (is_volume && !config.volume_keys_excluded)) {
        flags |= 1 << 0; // InputActionShowMediaFlyout
    }
    
    if config.lock_keys_enabled {
        match vk {
            0x14 => flags |= 1 << 1, // Caps
            0x90 => flags |= 1 << 2, // Num
            0x91 => flags |= 1 << 3, // Scroll
            0x2D if config.lock_insert_enabled => flags |= 1 << 4, // Insert
            _ => {}
        }
    }
    
    flags
}

#[unsafe(no_mangle)]
pub extern "C" fn should_handle_shell_appcommand_lparam(_lparam: isize) -> bool { true }
#[unsafe(no_mangle)]
pub extern "C" fn register_shell_hook_window(_hwnd: isize) -> bool { true }
#[unsafe(no_mangle)]
pub extern "C" fn deregister_shell_hook_window(_hwnd: isize) -> bool { true }

// --- Hook & Worker Logic ---

unsafe extern "system" fn keyboard_proc(n_code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    const WM_KEYDOWN: u32 = 0x0100;
    const WM_SYSKEYDOWN: u32 = 0x0104;

    if n_code >= 0 && (w_param.0 as u32 == WM_KEYDOWN || w_param.0 as u32 == WM_SYSKEYDOWN) {
        let kbd_struct = unsafe { *(l_param.0 as *const KBDLLHOOKSTRUCT) };
        if let Some(sender) = EVENT_SENDER.get() {
            let _ = sender.send(kbd_struct.vkCode as i32);
        }
    }
    
    let hook_guard = KEYBOARD_HOOK.lock().unwrap();
    let hhk = hook_guard.as_ref().map(|h| h.0).unwrap_or_default();
    unsafe { CallNextHookEx(Some(hhk), n_code, w_param, l_param) }
}

fn start_worker_thread() {
    if EVENT_SENDER.get().is_some() { return; }
    let (tx, rx) = mpsc::channel::<i32>();
    EVENT_SENDER.set(tx).ok();
    
    thread::spawn(move || {
        log_debug("Rust: Worker thread started");
        
        // Initialize Media Session Manager on background thread to avoid blocking UI
        log_debug("Rust: Background initializing Media Session Manager");
        match GlobalSystemMediaTransportControlsSessionManager::RequestAsync() {
            Ok(async_op) => {
                match async_op.get() {
                    Ok(m) => {
                        log_debug("Rust: Media Session Manager acquired on background thread");
                        MANAGER.set(m).ok();
                    }
                    Err(e) => {
                        log_debug(&format!("Rust: Failed to get manager on background: {:?}", e));
                    }
                }
            }
            Err(e) => {
                log_debug(&format!("Rust: RequestAsync failed on background: {:?}", e));
            }
        }

        while let Ok(vk_code) = rx.recv() {
            if let Some(callback) = CALLBACK.get() {
                callback(vk_code);
            }
        }
        log_debug("Rust: Worker thread stopped");
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn set_keyboard_hook(callback: extern "C" fn(i32)) -> bool {
    log_debug("Rust: set_keyboard_hook called");
    let mut hook_guard = KEYBOARD_HOOK.lock().unwrap();
    if hook_guard.is_some() { return true; }
    
    CALLBACK.set(callback).ok();
    start_worker_thread();

    unsafe {
        let mut h_module = GetModuleHandleW(w!("fluent_flyout_core.dll")).unwrap_or(HMODULE::default());
        if h_module.is_invalid() {
            set_last_error("Rust: fluent_flyout_core.dll handle failed, trying None");
            h_module = GetModuleHandleW(None).unwrap_or(HMODULE::default());
        }
        
        log_debug(&format!("Rust: DLL Module Handle: {:?}", h_module));
        
        match SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), Some(HINSTANCE(h_module.0)), 0) {
            Ok(h_hook) => {
                set_last_error("Rust: Keyboard Hook Set Successful");
                *hook_guard = Some(SendHandle(h_hook));
                true
            }
            Err(e) => {
                let err = windows::Win32::Foundation::GetLastError();
                set_last_error(&format!("Rust: SetWindowsHookExW Failed. Error={:?}, WinRS_Err={:?}", err, e));
                false
            }
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn unset_keyboard_hook() {
    log_debug("Rust: unset_keyboard_hook called");
    let mut hook_guard = KEYBOARD_HOOK.lock().unwrap();
    if let Some(h_hook) = hook_guard.take() {
        unsafe { let _ = UnhookWindowsHookEx(h_hook.0); }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn is_fullscreen_app_running() -> bool {
    unsafe {
        match SHQueryUserNotificationState() {
            Ok(state) => state == QUNS_RUNNING_D3D_FULL_SCREEN,
            Err(_) => false,
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn process_audio_fft(
    samples_ptr: *const f32,
    num_samples: usize,
    _sample_rate: u32,
    _bar_count: usize,
    fft_output_ptr: *mut f32,
    _min_db: f32,
    _max_db: f32,
) {
    if samples_ptr.is_null() || fft_output_ptr.is_null() || num_samples == 0 { return; }
    let samples = unsafe { std::slice::from_raw_parts(samples_ptr, num_samples) };
    let mut input: Vec<Complex<f32>> = samples.iter().map(|&s| Complex::new(s, 0.0)).collect();
    let planner = get_fft_planner();
    let mut planner_guard = planner.lock().unwrap();
    let fft = planner_guard.plan_fft_forward(num_samples);
    fft.process(&mut input);
    let output = unsafe { std::slice::from_raw_parts_mut(fft_output_ptr, num_samples / 2) };
    for i in 0..(num_samples / 2) { output[i] = input[i].norm(); }
}

#[unsafe(no_mangle)]
pub extern "C" fn draw_visualizer_bars(
    pixel_buffer: *mut u32,
    width: i32,
    height: i32,
    stride: i32,
    bar_values_ptr: *const f32,
    bar_count: usize,
    bar_spacing: i32,
    color_r: u8,
    color_g: u8,
    color_b: u8,
    centered_bars: bool,
    bar_baseline: i32,
) {
    if pixel_buffer.is_null() || bar_values_ptr.is_null() || bar_count == 0 { return; }
    let bar_values = unsafe { std::slice::from_raw_parts(bar_values_ptr, bar_count) };
    let color = (color_r as u32) << 16 | (color_g as u32) << 8 | (color_b as u32);
    let bar_width = (width - (bar_count as i32 - 1) * bar_spacing) / bar_count as i32;
    unsafe {
        for (i, &value) in bar_values.iter().enumerate() {
            let bar_height = (value * height as f32) as i32;
            let x_start = i as i32 * (bar_width + bar_spacing);
            let y_start = if centered_bars { (height - bar_height) / 2 } else { bar_baseline - bar_height };
            for y in y_start..(y_start + bar_height) {
                if y < 0 || y >= height { continue; }
                for x in x_start..(x_start + bar_width) {
                    if x < 0 || x >= width { continue; }
                    let idx = (y * (stride / 4) + x) as usize;
                    *pixel_buffer.add(idx) = color;
                }
            }
        }
    }
}
