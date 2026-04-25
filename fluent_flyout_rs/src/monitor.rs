use windows::Win32::Foundation::{LPARAM, POINT, RECT};
use windows::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, MonitorFromPoint, MonitorFromWindow, HDC, HMONITOR,
    MONITORENUMPROC, MONITORINFO, MONITORINFOEXW, MONITOR_DEFAULTTONEAREST,
};
use windows::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};
use windows::Win32::UI::WindowsAndMessaging::{GetCursorPos, GetForegroundWindow};

#[derive(Clone, Debug)]
pub struct MonitorInfo {
    pub left: i32,
    pub top: i32,
    pub width: i32,
    pub height: i32,
    pub monitor_left: i32,
    pub monitor_top: i32,
    pub monitor_width: i32,
    pub monitor_height: i32,
    pub is_primary: bool,
}

impl Default for MonitorInfo {
    fn default() -> Self {
        Self {
            left: 0,
            top: 0,
            width: 1920,
            height: 1040,
            monitor_left: 0,
            monitor_top: 0,
            monitor_width: 1920,
            monitor_height: 1080,
            is_primary: true,
        }
    }
}

pub fn get_selected_monitor(index: i32) -> MonitorInfo {
    let mut monitors = enumerate_monitors();
    if monitors.is_empty() {
        return MonitorInfo::default();
    }

    monitors.sort_by(|left, right| {
        right
            .is_primary
            .cmp(&left.is_primary)
            .then(left.monitor_left.cmp(&right.monitor_left))
            .then(left.monitor_top.cmp(&right.monitor_top))
    });

    let bounded_index = index.clamp(0, monitors.len().saturating_sub(1) as i32) as usize;
    monitors
        .into_iter()
        .nth(bounded_index)
        .unwrap_or_else(MonitorInfo::default)
}

pub fn get_lock_target_monitor(preference: i32, selected_index: i32) -> MonitorInfo {
    match preference {
        1 => get_monitor_with_focused_window(),
        2 => get_monitor_with_cursor(),
        _ => get_selected_monitor(selected_index),
    }
}

fn enumerate_monitors() -> Vec<MonitorInfo> {
    let mut monitors = Vec::new();

    unsafe {
        let callback: MONITORENUMPROC = Some(enum_monitor_callback);
        let lparam = LPARAM((&mut monitors as *mut Vec<MonitorInfo>) as isize);
        let _ = EnumDisplayMonitors(None, None, callback, lparam);
    }

    monitors
}

fn get_monitor_with_cursor() -> MonitorInfo {
    let mut cursor = POINT::default();
    if unsafe { GetCursorPos(&mut cursor) }.is_ok() {
        let hmonitor = unsafe { MonitorFromPoint(cursor, MONITOR_DEFAULTTONEAREST) };
        if let Some(info) = unsafe { read_monitor_info(hmonitor) } {
            return info;
        }
    }

    get_selected_monitor(0)
}

fn get_monitor_with_focused_window() -> MonitorInfo {
    let hwnd = unsafe { GetForegroundWindow() };
    if !hwnd.is_invalid() {
        let hmonitor = unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST) };
        if let Some(info) = unsafe { read_monitor_info(hmonitor) } {
            return info;
        }
    }

    get_monitor_with_cursor()
}

unsafe extern "system" fn enum_monitor_callback(
    hmonitor: HMONITOR,
    _hdc: HDC,
    _rect: *mut RECT,
    lparam: LPARAM,
) -> windows::core::BOOL {
    let monitors = unsafe { &mut *(lparam.0 as *mut Vec<MonitorInfo>) };

    if let Some(info) = unsafe { read_monitor_info(hmonitor) } {
        monitors.push(info);
    }

    windows::core::BOOL(1)
}

unsafe fn read_monitor_info(hmonitor: HMONITOR) -> Option<MonitorInfo> {
    let mut info = MONITORINFOEXW::default();
    info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;

    if !unsafe { GetMonitorInfoW(hmonitor, &mut info as *mut MONITORINFOEXW as *mut MONITORINFO) }
        .as_bool()
    {
        return None;
    }

    let work_area = info.monitorInfo.rcWork;
    let monitor_area = info.monitorInfo.rcMonitor;
    let mut dpi_x = 96;
    let mut dpi_y = 96;
    let _ = unsafe { GetDpiForMonitor(hmonitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y) };

    Some(MonitorInfo {
        left: work_area.left,
        top: work_area.top,
        width: work_area.right - work_area.left,
        height: work_area.bottom - work_area.top,
        monitor_left: monitor_area.left,
        monitor_top: monitor_area.top,
        monitor_width: monitor_area.right - monitor_area.left,
        monitor_height: monitor_area.bottom - monitor_area.top,
        is_primary: (info.monitorInfo.dwFlags & 1) != 0,
    })
}
