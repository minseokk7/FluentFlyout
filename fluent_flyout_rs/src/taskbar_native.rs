use crate::monitor;
use windows::core::w;
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::UI::HiDpi::GetDpiForWindow;
use windows::Win32::UI::WindowsAndMessaging::{
    FindWindowExW, FindWindowW, GetWindowLongPtrW, GetWindowRect, SetParent, SetWindowLongPtrW,
    SetWindowPos, GWL_STYLE, SWP_ASYNCWINDOWPOS, SWP_NOACTIVATE, SWP_NOZORDER, SWP_SHOWWINDOW,
    WS_CHILD, WS_POPUP,
};

const ORIGINAL_LEFT_PADDING: i32 = 20;
const ORIGINAL_WIDGETS_PADDING: i32 = 216;
const ORIGINAL_TRAY_GAP: i32 = 1;
const ORIGINAL_WIDGET_GAP: i32 = 2;

pub fn place_widget_in_taskbar(
    widget_hwnd: HWND,
    selected_monitor: i32,
    position: i32,
    auto_padding: bool,
    manual_padding: i32,
    widget_width: i32,
    widget_height: i32,
) -> bool {
    let Some((taskbar_hwnd, is_main_taskbar)) = find_taskbar_for_monitor(selected_monitor) else {
        return false;
    };

    let Some(taskbar_rect) = window_rect(taskbar_hwnd) else {
        return false;
    };

    let taskbar_width = taskbar_rect.right - taskbar_rect.left;
    let taskbar_height = taskbar_rect.bottom - taskbar_rect.top;
    if taskbar_width <= 0 || taskbar_height <= 0 {
        return false;
    }

    attach_as_taskbar_child(widget_hwnd, taskbar_hwnd);

    let dpi_scale = (unsafe { GetDpiForWindow(taskbar_hwnd) } as f64 / 96.0).max(1.0);
    let width = widget_width.clamp(80, taskbar_width.max(80));
    let height = widget_height
        .min((40.0 * dpi_scale).round() as i32)
        .clamp(32, taskbar_height.max(32));
    let y = ((taskbar_height - height) / 2).max(0);
    let x = position_widget_x(
        taskbar_hwnd,
        taskbar_rect,
        is_main_taskbar,
        position,
        auto_padding,
        manual_padding,
        width,
    )
    .clamp(0, (taskbar_width - width).max(0));

    unsafe {
        SetWindowPos(
            widget_hwnd,
            None,
            x,
            y,
            width,
            height,
            SWP_NOZORDER | SWP_NOACTIVATE | SWP_ASYNCWINDOWPOS | SWP_SHOWWINDOW,
        )
        .is_ok()
    }
}

fn attach_as_taskbar_child(widget_hwnd: HWND, taskbar_hwnd: HWND) {
    unsafe {
        let style = GetWindowLongPtrW(widget_hwnd, GWL_STYLE) as u32;
        let child_style = (style & !WS_POPUP.0) | WS_CHILD.0;
        SetWindowLongPtrW(widget_hwnd, GWL_STYLE, child_style as isize);
        let _ = SetParent(widget_hwnd, Some(taskbar_hwnd));
    }
}

fn position_widget_x(
    taskbar_hwnd: HWND,
    taskbar_rect: RECT,
    is_main_taskbar: bool,
    position: i32,
    auto_padding: bool,
    manual_padding: i32,
    widget_width: i32,
) -> i32 {
    let taskbar_width = taskbar_rect.right - taskbar_rect.left;
    let base = match position {
        0 => {
            let mut left = ORIGINAL_LEFT_PADDING;
            if auto_padding {
                // Original fallback when UI Automation cannot read the native WidgetsButton rect.
                left += ORIGINAL_WIDGETS_PADDING + ORIGINAL_WIDGET_GAP;
            }
            left
        }
        1 => (taskbar_width - widget_width) / 2,
        2 => {
            if is_main_taskbar {
                tray_left(taskbar_hwnd, taskbar_rect)
                    .map(|left| left - taskbar_rect.left - widget_width - ORIGINAL_TRAY_GAP)
                    .unwrap_or(taskbar_width - widget_width - ORIGINAL_LEFT_PADDING)
            } else {
                // Secondary taskbars often do not expose TrayNotifyWnd; original falls back through UIA.
                taskbar_width - widget_width - ORIGINAL_LEFT_PADDING
            }
        }
        _ => ORIGINAL_LEFT_PADDING,
    };

    base + manual_padding
}

fn tray_left(taskbar_hwnd: HWND, taskbar_rect: RECT) -> Option<i32> {
    let tray = unsafe { FindWindowExW(Some(taskbar_hwnd), None, w!("TrayNotifyWnd"), None).ok()? };
    let tray_rect = window_rect(tray)?;
    if tray_rect.left <= taskbar_rect.left || tray_rect.left >= taskbar_rect.right {
        return None;
    }
    Some(tray_rect.left)
}

fn find_taskbar_for_monitor(selected_monitor: i32) -> Option<(HWND, bool)> {
    let selected = monitor::get_selected_monitor(selected_monitor);
    let primary = unsafe { FindWindowW(w!("Shell_TrayWnd"), None).ok() };

    if let Some(primary_hwnd) = primary {
        if taskbar_matches_monitor(primary_hwnd, &selected) {
            return Some((primary_hwnd, true));
        }
    }

    let mut after = None;
    while let Ok(hwnd) = unsafe { FindWindowExW(None, after, w!("Shell_SecondaryTrayWnd"), None) } {
        if taskbar_matches_monitor(hwnd, &selected) {
            return Some((hwnd, false));
        }
        after = Some(hwnd);
    }

    primary.map(|hwnd| (hwnd, true))
}

fn taskbar_matches_monitor(hwnd: HWND, selected: &monitor::MonitorInfo) -> bool {
    let Some(rect) = window_rect(hwnd) else {
        return false;
    };

    let center_x = rect.left + (rect.right - rect.left) / 2;
    let center_y = rect.top + (rect.bottom - rect.top) / 2;
    center_x >= selected.monitor_left
        && center_x < selected.monitor_left + selected.monitor_width
        && center_y >= selected.monitor_top
        && center_y < selected.monitor_top + selected.monitor_height
}

fn window_rect(hwnd: HWND) -> Option<RECT> {
    let mut rect = RECT::default();
    unsafe { GetWindowRect(hwnd, &mut rect) }.ok()?;
    Some(rect)
}
