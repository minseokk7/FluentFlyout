#[cfg(windows)]
mod imp {
    use windows::Win32::Foundation::{LPARAM, RECT};
    use windows::Win32::Graphics::Gdi::{EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFO, MONITORINFOEXW};

    use crate::models::MonitorDto;

    pub fn list_monitors() -> Vec<MonitorDto> {
        let mut monitors = Vec::new();
        unsafe {
            let lparam = LPARAM((&mut monitors as *mut Vec<MonitorDto>) as isize);
            let _ = EnumDisplayMonitors(None, None, Some(enum_monitor_callback), lparam);
        }
        if monitors.is_empty() {
            monitors.push(MonitorDto { id: 0, name: "* 1 (Generic PnP Monitor)".to_string(), left: 0, top: 0, width: 1920, height: 1080, is_primary: true });
        }
        monitors.sort_by(|left, right| right.is_primary.cmp(&left.is_primary).then(left.left.cmp(&right.left)).then(left.top.cmp(&right.top)));
        for (index, monitor) in monitors.iter_mut().enumerate() {
            monitor.id = index as i32;
            monitor.name = if monitor.is_primary { format!("* {} (Generic PnP Monitor)", index + 1) } else { format!("{} (Generic PnP Monitor)", index + 1) };
        }
        monitors
    }

    pub fn selected_monitor(index: i32) -> MonitorDto {
        let monitors = list_monitors();
        let selected = index.clamp(0, monitors.len().saturating_sub(1) as i32) as usize;
        monitors.get(selected).cloned().unwrap_or_else(|| MonitorDto { id: 0, name: "* 1 (Generic PnP Monitor)".to_string(), left: 0, top: 0, width: 1920, height: 1080, is_primary: true })
    }

    unsafe extern "system" fn enum_monitor_callback(hmonitor: HMONITOR, _hdc: HDC, _rect: *mut RECT, lparam: LPARAM) -> windows::core::BOOL {
        let monitors = unsafe { &mut *(lparam.0 as *mut Vec<MonitorDto>) };
        let mut info = MONITORINFOEXW::default();
        info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;
        if unsafe { GetMonitorInfoW(hmonitor, &mut info as *mut MONITORINFOEXW as *mut MONITORINFO) }.as_bool() {
            let monitor = info.monitorInfo.rcMonitor;
            monitors.push(MonitorDto {
                id: monitors.len() as i32,
                name: String::new(),
                left: monitor.left,
                top: monitor.top,
                width: monitor.right - monitor.left,
                height: monitor.bottom - monitor.top,
                is_primary: (info.monitorInfo.dwFlags & 1) != 0,
            });
        }
        windows::core::BOOL(1)
    }
}

#[cfg(not(windows))]
mod imp {
    use crate::models::MonitorDto;

    pub fn list_monitors() -> Vec<MonitorDto> {
        vec![MonitorDto { id: 0, name: "* 1 (Generic PnP Monitor)".to_string(), left: 0, top: 0, width: 1920, height: 1080, is_primary: true }]
    }

    pub fn selected_monitor(_index: i32) -> MonitorDto { list_monitors().remove(0) }
}

pub use imp::{list_monitors, selected_monitor};