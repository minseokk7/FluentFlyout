use crate::{
    error::{AppError, AppResult},
    models::{MediaSessionDto, SettingsDto, TaskbarWidgetPlacementDto},
    services::monitor,
};

const ORIGINAL_LEFT_PADDING: i32 = 20;
const ORIGINAL_TRAY_GAP: i32 = 1;
const DEFAULT_LOGICAL_WIDTH: i32 = 100;
const MIN_CONTROLS_LOGICAL_WIDTH: i32 = 170;
const MAX_NATIVE_WIDGET_PADDING: i32 = 216;
const CONTROLS_LOGICAL_WIDTH: i32 = 102;
const DEFAULT_LOGICAL_HEIGHT: i32 = 40;
const ART_LOGICAL_SIZE: i32 = 36;
const INFO_MARGIN: i32 = 8;
const ART_AND_TEXT_MARGIN: i32 = 55;
const VISUALIZER_BAR_WIDTH: i32 = 5;
const VISUALIZER_BAR_GAP: i32 = 4;
const VISUALIZER_SIDE_PADDING: i32 = 10;
const VISUALIZER_GAP: i32 = 8;
const ORIGINAL_WIDGET_SCALE: f64 = 0.9;
const TIMER_ID: usize = 1;
const TIMER_INTERVAL_MS: u32 = 1500;

pub fn calculate_placement(settings: &SettingsDto) -> AppResult<TaskbarWidgetPlacementDto> {
    let selected = monitor::selected_monitor(settings.taskbar_widget_selected_monitor);
    let taskbar = taskbar_rect_for_monitor(&selected, settings.legacy_taskbar_width_enabled)
        .ok_or_else(|| AppError::new("taskbar.parent", "작업표시줄 창을 찾을 수 없습니다."))?;

    let scale = taskbar.dpi_scale;
    let logical_width = dynamic_logical_width()
        + if settings.taskbar_widget_controls_enabled {
            CONTROLS_LOGICAL_WIDTH
        } else {
            0
        };
    let logical_width = if settings.taskbar_widget_controls_enabled {
        logical_width.max(MIN_CONTROLS_LOGICAL_WIDTH + CONTROLS_LOGICAL_WIDTH)
    } else {
        logical_width
    };
    let rendered_logical_width = ((logical_width as f64 * ORIGINAL_WIDGET_SCALE).round() as i32).max(40);
    let width = ((rendered_logical_width as f64 * scale).round() as i32)
        .clamp(80, taskbar.width.max(80));
    let height = ((DEFAULT_LOGICAL_HEIGHT as f64 * scale).round() as i32)
        .clamp(28, taskbar.height.max(28));
    let widget_y = ((taskbar.height - height) / 2).max(0);

    let mut widget_x = match settings.taskbar_widget_position {
        0 => left_position(&taskbar, settings),
        1 => (taskbar.width - width) / 2,
        2 => right_position(&taskbar, settings, width),
        _ => ORIGINAL_LEFT_PADDING,
    };
    widget_x += settings.taskbar_widget_manual_padding;
    widget_x = widget_x.clamp(0, (taskbar.width - width).max(0));

    Ok(TaskbarWidgetPlacementDto {
        monitor_id: selected.id,
        taskbar_hwnd: taskbar.hwnd,
        x: taskbar.left + widget_x,
        y: taskbar.top + widget_y,
        width,
        height,
        dpi_scale: scale,
        logical_width: rendered_logical_width,
        logical_height: DEFAULT_LOGICAL_HEIGHT,
        container_x: taskbar.left,
        container_y: taskbar.top,
        container_width: taskbar.width,
        container_height: taskbar.height,
        widget_x,
        widget_y,
        source: taskbar.source,
    })
}

pub fn apply_settings_on_main_thread<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    settings: SettingsDto,
) -> AppResult<TaskbarWidgetPlacementDto> {
    let placement = calculate_placement(&settings)?;
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    let settings_for_thread = settings.clone();
    let placement_for_thread = placement.clone();
    let app_for_thread = app.clone();

    app.run_on_main_thread(move || {
        let result = apply_settings(&app_for_thread, &settings_for_thread, &placement_for_thread);
        let _ = tx.send(result);
    })
    .map_err(|error| {
        AppError::with_detail(
            "taskbar.main_thread",
            "작업표시줄 위젯 작업을 메인 스레드에 예약하지 못했습니다.",
            error,
        )
    })?;

    rx.recv()
        .map_err(|error| {
            AppError::with_detail(
                "taskbar.main_thread.recv",
                "작업표시줄 위젯 작업 결과를 받을 수 없습니다.",
                error,
            )
        })??;
    Ok(placement)
}

pub fn destroy_widget_on_main_thread<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    let app_for_thread = app.clone();
    let _ = app.run_on_main_thread(move || {
        let _ = destroy_native_widget(&app_for_thread);
    });
}

pub fn apply_initial_settings<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    settings: &SettingsDto,
) -> AppResult<TaskbarWidgetPlacementDto> {
    destroy_orphan_widgets();
    let placement = calculate_placement(settings)?;
    apply_settings(app, settings, &placement)?;
    Ok(placement)
}

pub fn destroy_orphan_widgets() {
    destroy_orphan_widgets_impl();
}

fn dynamic_logical_width() -> i32 {
    #[cfg(windows)]
    {
        if let Ok(draw) = draw_state().lock() {
            let text_width = estimate_text_width(&draw.title).max(estimate_text_width(&draw.artist));
            let measured = text_width + ART_AND_TEXT_MARGIN;
            return measured
                .max(DEFAULT_LOGICAL_WIDTH)
                .min((MAX_NATIVE_WIDGET_PADDING as f64 / ORIGINAL_WIDGET_SCALE).round() as i32);
        }
    }

    DEFAULT_LOGICAL_WIDTH
}

fn estimate_text_width(text: &str) -> i32 {
    let mut width: f64 = 0.0;
    for ch in text.chars().take(80) {
        width += if ch.is_ascii() {
            if ch.is_ascii_whitespace() { 4.0 } else { 7.2 }
        } else {
            12.0
        };
    }
    width.round() as i32
}

#[cfg(windows)]
#[derive(Clone, Copy, Debug)]
pub enum TaskbarWidgetAction {
    ShowFlyout,
    Previous,
    PlayPause,
    Next,
}

#[cfg(not(windows))]
#[derive(Clone, Copy, Debug)]
pub enum TaskbarWidgetAction {
    ShowFlyout,
}

pub fn set_click_sender(sender: std::sync::mpsc::Sender<TaskbarWidgetAction>) {
    set_click_sender_impl(sender);
}

pub fn update_media(media: &MediaSessionDto) {
    update_media_impl(media);
}

pub fn update_audio_peak(peak: f32) {
    update_audio_peak_impl(peak);
}

fn left_position(taskbar: &TaskbarRect, settings: &SettingsDto) -> i32 {
    if !settings.taskbar_widget_padding {
        return ORIGINAL_LEFT_PADDING;
    }

    taskbar
        .widgets_rect
        .as_ref()
        .map(|rect| rect.right - taskbar.left + 2)
        .unwrap_or(ORIGINAL_LEFT_PADDING)
}

fn right_position(taskbar: &TaskbarRect, settings: &SettingsDto, width: i32) -> i32 {
    if settings.taskbar_widget_padding {
        if let Some(widgets) = &taskbar.widgets_rect {
            if widgets.left > taskbar.left + taskbar.width / 2 {
                return widgets.left - taskbar.left - width - ORIGINAL_TRAY_GAP;
            }
        }
    }

    // 원본 앱은 오른쪽 배치에서 시스템 트레이 인접 배치를 기본으로 사용합니다.
    // UIA SystemTrayIcon은 DPI 가상화 환경에서 좌표계가 섞일 수 있어 보조 fallback으로만 사용합니다.
    if let Some(tray) = &taskbar.tray_rect {
        return tray.left - taskbar.left - width - ORIGINAL_TRAY_GAP;
    }

    if let Some(system_tray) = &taskbar.system_tray_rect {
        return system_tray.left - taskbar.left - width - ORIGINAL_TRAY_GAP;
    }

    taskbar.width - width - ORIGINAL_LEFT_PADDING
}

fn visualizer_rect(
    settings: &SettingsDto,
    placement: &TaskbarWidgetPlacementDto,
) -> Option<PhysicalRect> {
    if !settings.taskbar_visualizer_enabled {
        return None;
    }

    let bar_count = settings.taskbar_visualizer_bar_count.clamp(1, 20);
    let width =
        VISUALIZER_SIDE_PADDING * 2 + bar_count * VISUALIZER_BAR_WIDTH + (bar_count - 1) * VISUALIZER_BAR_GAP;
    let height = placement.height.clamp(24, placement.container_height.max(24));
    let top = ((placement.container_height - height) / 2).max(0);
    let left = if settings.taskbar_visualizer_position == 0 {
        placement.widget_x - width - VISUALIZER_GAP
    } else {
        placement.widget_x + placement.width + VISUALIZER_GAP
    }
    .clamp(0, (placement.container_width - width).max(0));

    Some(PhysicalRect {
        left,
        top,
        right: left + width,
        bottom: top + height,
    })
}

#[derive(Clone, Debug)]
struct TaskbarRect {
    hwnd: String,
    left: i32,
    top: i32,
    width: i32,
    height: i32,
    dpi_scale: f64,
    widgets_rect: Option<PhysicalRect>,
    system_tray_rect: Option<PhysicalRect>,
    tray_rect: Option<PhysicalRect>,
    source: String,
}

#[derive(Clone, Debug)]
struct PhysicalRect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

#[cfg(windows)]
fn taskbar_rect_for_monitor(
    selected: &crate::models::MonitorDto,
    legacy_width_enabled: bool,
) -> Option<TaskbarRect> {
    use windows::core::w;
    use windows::Win32::Foundation::{HWND, RECT};
    use windows::Win32::UI::HiDpi::GetDpiForWindow;
    use windows::Win32::UI::WindowsAndMessaging::{FindWindowExW, FindWindowW, GetWindowRect};

    fn window_rect(hwnd: HWND) -> Option<RECT> {
        let mut rect = RECT::default();
        unsafe { GetWindowRect(hwnd, &mut rect) }.ok()?;
        Some(rect)
    }

    fn intersects_monitor(rect: &PhysicalRect, selected: &crate::models::MonitorDto) -> bool {
        let left = rect.left.max(selected.left);
        let top = rect.top.max(selected.top);
        let right = rect.right.min(selected.left + selected.width);
        let bottom = rect.bottom.min(selected.top + selected.height);
        right > left && bottom > top
    }

    fn taskbar_if_matches(
        hwnd: HWND,
        selected: &crate::models::MonitorDto,
        source: &str,
        legacy_width_enabled: bool,
    ) -> Option<TaskbarRect> {
        let shell_rect = PhysicalRect::from_rect(window_rect(hwnd)?);
        let taskbar_frame = if legacy_width_enabled {
            None
        } else {
            uia_rect(hwnd, "TaskbarFrame")
        };
        let selected_rect = taskbar_frame.clone().unwrap_or_else(|| shell_rect.clone());
        if selected_rect.width() <= 0
            || selected_rect.height() <= 0
            || !intersects_monitor(&selected_rect, selected)
        {
            return None;
        }

        let dpi_scale = unsafe { GetDpiForWindow(hwnd) } as f64 / 96.0;
        let widgets_rect = uia_rect(hwnd, "WidgetsButton");
        let system_tray_rect = uia_rect(hwnd, "SystemTrayIcon");
        let tray_rect = unsafe { FindWindowExW(Some(hwnd), None, w!("TrayNotifyWnd"), None).ok() }
            .and_then(window_rect)
            .map(PhysicalRect::from_rect);

        let mut final_source = source.to_string();
        if taskbar_frame.is_some() {
            final_source = format!("{final_source}:uia-taskbar-frame");
        } else if tray_rect.is_some() {
            final_source = format!("{final_source}:tray-fallback");
        }

        Some(TaskbarRect {
            hwnd: format!("0x{:X}", hwnd.0 as usize),
            left: selected_rect.left,
            top: selected_rect.top,
            width: selected_rect.width(),
            height: selected_rect.height(),
            dpi_scale,
            widgets_rect,
            system_tray_rect,
            tray_rect,
            source: final_source,
        })
    }

    let primary = unsafe { FindWindowW(w!("Shell_TrayWnd"), None).ok() };
    let mut taskbar =
        primary.and_then(|hwnd| taskbar_if_matches(hwnd, selected, "shell-tray", legacy_width_enabled));

    if taskbar.is_none() {
        let mut after: Option<HWND> = None;
        while let Ok(hwnd) =
            unsafe { FindWindowExW(None, after, w!("Shell_SecondaryTrayWnd"), None) }
        {
            if let Some(rect) =
                taskbar_if_matches(hwnd, selected, "secondary-tray", legacy_width_enabled)
            {
                taskbar = Some(rect);
                break;
            }
            after = Some(hwnd);
        }
    }

    taskbar
}

#[cfg(not(windows))]
fn taskbar_rect_for_monitor(
    _selected: &crate::models::MonitorDto,
    _legacy_width_enabled: bool,
) -> Option<TaskbarRect> {
    None
}

#[cfg(windows)]
impl PhysicalRect {
    fn from_rect(rect: windows::Win32::Foundation::RECT) -> Self {
        Self {
            left: rect.left,
            top: rect.top,
            right: rect.right,
            bottom: rect.bottom,
        }
    }
}

impl PhysicalRect {
    fn width(&self) -> i32 {
        self.right - self.left
    }

    fn height(&self) -> i32 {
        self.bottom - self.top
    }
}

#[cfg(windows)]
fn uia_rect(hwnd: windows::Win32::Foundation::HWND, automation_id: &str) -> Option<PhysicalRect> {
    use windows::core::BSTR;
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED,
    };
    use windows::Win32::System::Variant::VARIANT;
    use windows::Win32::UI::Accessibility::{
        CUIAutomation8, IUIAutomation, TreeScope_Descendants, UIA_AutomationIdPropertyId,
    };

    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let automation: IUIAutomation =
            CoCreateInstance(&CUIAutomation8, None, CLSCTX_INPROC_SERVER).ok()?;
        let root = automation.ElementFromHandle(hwnd).ok()?;
        let value: VARIANT = BSTR::from(automation_id).into();
        let condition = automation
            .CreatePropertyCondition(UIA_AutomationIdPropertyId, &value)
            .ok()?;
        let element = root.FindFirst(TreeScope_Descendants, &condition).ok()?;
        let rect = element.CurrentBoundingRectangle().ok()?;
        let physical = PhysicalRect {
            left: rect.left as i32,
            top: rect.top as i32,
            right: rect.right as i32,
            bottom: rect.bottom as i32,
        };
        (physical.width() > 0 && physical.height() > 0).then_some(physical)
    }
}

#[cfg(windows)]
#[derive(Clone)]
struct NativeWidget {
    hwnd: isize,
    parent: isize,
    last_placement: Option<TaskbarWidgetPlacementDto>,
    managed_window: bool,
}

#[cfg(windows)]
#[derive(Clone)]
struct DrawState {
    title: String,
    artist: String,
    has_media: bool,
    playing: bool,
    audio_peak: f32,
    controls_enabled: bool,
    controls_position: i32,
    visualizer_enabled: bool,
    visualizer_bar_count: i32,
    visualizer_rect: Option<PhysicalRect>,
    placement: Option<TaskbarWidgetPlacementDto>,
}

#[cfg(windows)]
static WIDGET: std::sync::OnceLock<std::sync::Mutex<Option<NativeWidget>>> =
    std::sync::OnceLock::new();
#[cfg(windows)]
static DRAW_STATE: std::sync::OnceLock<std::sync::Mutex<DrawState>> =
    std::sync::OnceLock::new();
#[cfg(windows)]
static CLICK_SENDER: std::sync::OnceLock<std::sync::Mutex<Option<std::sync::mpsc::Sender<TaskbarWidgetAction>>>> =
    std::sync::OnceLock::new();

#[cfg(windows)]
fn widget_state() -> &'static std::sync::Mutex<Option<NativeWidget>> {
    WIDGET.get_or_init(|| std::sync::Mutex::new(None))
}

#[cfg(windows)]
fn draw_state() -> &'static std::sync::Mutex<DrawState> {
    DRAW_STATE.get_or_init(|| {
        std::sync::Mutex::new(DrawState {
            title: String::new(),
            artist: String::new(),
            has_media: false,
            playing: false,
            audio_peak: 0.0,
            controls_enabled: true,
            controls_position: 1,
            visualizer_enabled: false,
            visualizer_bar_count: 10,
            visualizer_rect: None,
            placement: None,
        })
    })
}

#[cfg(windows)]
fn click_sender() -> &'static std::sync::Mutex<Option<std::sync::mpsc::Sender<TaskbarWidgetAction>>> {
    CLICK_SENDER.get_or_init(|| std::sync::Mutex::new(None))
}

#[cfg(windows)]
fn set_click_sender_impl(sender: std::sync::mpsc::Sender<TaskbarWidgetAction>) {
    if let Ok(mut guard) = click_sender().lock() {
        *guard = Some(sender);
    }
}

#[cfg(not(windows))]
fn set_click_sender_impl(_sender: std::sync::mpsc::Sender<TaskbarWidgetAction>) {}

#[cfg(windows)]
fn update_media_impl(media: &MediaSessionDto) {
    let mut changed = false;
    if let Ok(mut draw) = draw_state().lock() {
        let has_media = !media.title.trim().is_empty() && media.title != "No media playing";
        let title = if has_media { media.title.clone() } else { String::new() };
        let artist = if has_media { media.artist.clone() } else { String::new() };
        let playing = media.playback_status == "playing";
        changed = draw.has_media != has_media
            || draw.title != title
            || draw.artist != artist
            || draw.playing != playing;
        if changed {
            draw.has_media = has_media;
            draw.title = title;
            draw.artist = artist;
            draw.playing = playing;
        }
    }

    if changed && let Ok(guard) = widget_state().lock() {
        if let Some(widget) = guard.as_ref() {
            invalidate_hwnd(widget.hwnd);
        }
    }
}

#[cfg(not(windows))]
fn update_media_impl(_media: &MediaSessionDto) {}

#[cfg(windows)]
fn update_audio_peak_impl(peak: f32) {
    let mut changed = false;
    if let Ok(mut draw) = draw_state().lock() {
        let peak = peak.clamp(0.0, 1.0);
        changed = (draw.audio_peak - peak).abs() > 0.03;
        if changed {
            draw.audio_peak = peak;
        }
    }

    if changed && let Ok(guard) = widget_state().lock() {
        if let Some(widget) = guard.as_ref() {
            invalidate_hwnd(widget.hwnd);
        }
    }
}

#[cfg(not(windows))]
fn update_audio_peak_impl(_peak: f32) {}

#[cfg(windows)]
fn apply_settings<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    settings: &SettingsDto,
    placement: &TaskbarWidgetPlacementDto,
) -> AppResult<()> {
    let hide_for_no_media = draw_state()
        .lock()
        .ok()
        .is_some_and(|draw| settings.taskbar_widget_hide_completely && !draw.has_media);

    if !settings.taskbar_widget_enabled || hide_for_no_media {
        return destroy_native_widget(app);
    }

    let parent = parse_hwnd(&placement.taskbar_hwnd)
        .ok_or_else(|| AppError::new("taskbar.parent", "작업표시줄 창 핸들을 확인할 수 없습니다."))?;

    {
        let mut draw = draw_state()
            .lock()
            .map_err(|_| AppError::new("taskbar.state", "작업표시줄 위젯 상태 잠금에 실패했습니다."))?;
        draw.controls_enabled = settings.taskbar_widget_controls_enabled;
        draw.controls_position = settings.taskbar_widget_controls_position;
        draw.visualizer_enabled = settings.taskbar_visualizer_enabled;
        draw.visualizer_bar_count = settings.taskbar_visualizer_bar_count.clamp(1, 20);
        draw.visualizer_rect = visualizer_rect(settings, placement);
        draw.placement = Some(placement.clone());
    }

    let mut guard = widget_state()
        .lock()
        .map_err(|_| AppError::new("taskbar.state", "작업표시줄 위젯 상태 잠금에 실패했습니다."))?;
    let needs_create = guard
        .as_ref()
        .map_or(true, |widget| widget.parent != parent.0 as isize);

    if needs_create {
        if let Some(widget) = guard.take() {
            dispose_widget(app, widget);
        }
        let hwnd = create_native_widget(app, parent)?;
        *guard = Some(NativeWidget {
            hwnd: hwnd.0 as isize,
            parent: parent.0 as isize,
            last_placement: None,
            managed_window: false,
        });
    }

    let Some(widget) = guard.as_mut() else {
        return Err(AppError::new("taskbar.create", "작업표시줄 위젯 창을 만들 수 없습니다."));
    };

    let placement_changed = widget
        .last_placement
        .as_ref()
        .is_none_or(|current| current != placement);
    if placement_changed {
        position_native_widget(widget, placement)?;
        widget.last_placement = Some(placement.clone());
    }
    Ok(())
}

#[cfg(not(windows))]
fn apply_settings<R: tauri::Runtime>(
    _app: &tauri::AppHandle<R>,
    _settings: &SettingsDto,
    _placement: &TaskbarWidgetPlacementDto,
) -> AppResult<()> {
    Ok(())
}

#[cfg(windows)]
fn create_native_widget<R: tauri::Runtime>(
    _app: &tauri::AppHandle<R>,
    parent: windows::Win32::Foundation::HWND,
) -> AppResult<windows::Win32::Foundation::HWND> {
    use windows::core::w;
    use windows::Win32::Foundation::{GetLastError, ERROR_CLASS_ALREADY_EXISTS};
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, RegisterClassW, SetParent, SetTimer, ShowWindow, CS_HREDRAW,
        CS_VREDRAW, HMENU, SW_SHOWNA, WINDOW_EX_STYLE, WNDCLASSW, WS_CHILD, WS_EX_NOACTIVATE,
        WS_EX_TOOLWINDOW, WS_VISIBLE,
    };

    unsafe {
        let module = GetModuleHandleW(None).map_err(|error| {
            AppError::with_detail(
                "taskbar.module",
                "작업표시줄 위젯 모듈 핸들을 가져오지 못했습니다.",
                error,
            )
        })?;
        let instance = windows::Win32::Foundation::HINSTANCE(module.0);
        let class_name = w!("FluentFlyoutRustTaskbarWidget");
        let class = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(widget_wnd_proc),
            hInstance: instance,
            lpszClassName: class_name,
            ..Default::default()
        };

        let atom = RegisterClassW(&class);
        if atom == 0 && GetLastError() != ERROR_CLASS_ALREADY_EXISTS {
            return Err(AppError::new(
                "taskbar.register_class",
                "작업표시줄 위젯 창 클래스를 등록하지 못했습니다.",
            ));
        }

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE(WS_EX_NOACTIVATE.0 | WS_EX_TOOLWINDOW.0),
            class_name,
            w!("FluentFlyoutRustTaskbarWidget"),
            WS_CHILD | WS_VISIBLE,
            0,
            0,
            1,
            1,
            Some(parent),
            Some(HMENU(std::ptr::null_mut())),
            Some(instance),
            None,
        )
        .map_err(|error| {
            AppError::with_detail(
                "taskbar.create_window",
                "작업표시줄 위젯 네이티브 창을 만들지 못했습니다.",
                error,
            )
        })?;

        let _ = SetParent(hwnd, Some(parent));
        let _ = SetTimer(Some(hwnd), TIMER_ID, TIMER_INTERVAL_MS, None);
        let _ = ShowWindow(hwnd, SW_SHOWNA);
        Ok(hwnd)
    }
}

#[cfg(windows)]
fn position_native_widget(widget: &NativeWidget, placement: &TaskbarWidgetPlacementDto) -> AppResult<()> {
    use windows::Win32::Foundation::{HWND, POINT};
    use windows::Win32::Graphics::Gdi::{
        CombineRgn, CreateRectRgn, DeleteObject, InvalidateRect, ScreenToClient, SetWindowRgn,
        UpdateWindow, RGN_OR,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        SetWindowPos, HWND_TOP, SWP_ASYNCWINDOWPOS, SWP_NOACTIVATE, SWP_SHOWWINDOW,
    };

    let hwnd = HWND(widget.hwnd as *mut std::ffi::c_void);
    let parent = HWND(widget.parent as *mut std::ffi::c_void);
    unsafe {
        let mut container_pos = POINT {
            x: placement.container_x,
            y: placement.container_y,
        };
        let _ = ScreenToClient(parent, &mut container_pos);
        SetWindowPos(
            hwnd,
            Some(HWND_TOP),
            container_pos.x,
            container_pos.y,
            placement.container_width,
            placement.container_height,
            SWP_NOACTIVATE | SWP_ASYNCWINDOWPOS | SWP_SHOWWINDOW,
        )
        .map_err(|error| {
            AppError::with_detail(
                "taskbar.set_window_pos",
                "작업표시줄 위젯 위치 적용에 실패했습니다.",
                error,
            )
        })?;

        let region = CreateRectRgn(
            placement.widget_x,
            placement.widget_y,
            placement.widget_x + placement.width,
            placement.widget_y + placement.height,
        );
        if region.is_invalid() {
            return Err(AppError::new(
                "taskbar.region",
                "작업표시줄 위젯 영역 적용에 실패했습니다.",
            ));
        }

        if let Some(visualizer) = draw_state()
            .lock()
            .ok()
            .and_then(|draw| draw.visualizer_rect.clone())
        {
            let visualizer_region = CreateRectRgn(
                visualizer.left,
                visualizer.top,
                visualizer.right,
                visualizer.bottom,
            );
            if !visualizer_region.is_invalid() {
                let _ = CombineRgn(Some(region), Some(region), Some(visualizer_region), RGN_OR);
                let _ = DeleteObject(visualizer_region.into());
            }
        }

        let _ = SetWindowRgn(hwnd, Some(region), true);
        keep_child_widget_on_top(hwnd);
        let _ = InvalidateRect(Some(hwnd), None, false);
        let _ = UpdateWindow(hwnd);
    }
    Ok(())
}

#[cfg(windows)]
fn destroy_native_widget<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> AppResult<()> {
    let mut guard = widget_state()
        .lock()
        .map_err(|_| AppError::new("taskbar.state", "작업표시줄 위젯 상태 잠금에 실패했습니다."))?;
    if let Some(widget) = guard.take() {
        dispose_widget(app, widget);
    }
    Ok(())
}

#[cfg(not(windows))]
fn destroy_native_widget<R: tauri::Runtime>(_app: &tauri::AppHandle<R>) -> AppResult<()> {
    Ok(())
}

#[cfg(windows)]
fn destroy_hwnd(hwnd_raw: isize) {
    use windows::Win32::Foundation::HWND;

    let hwnd = HWND(hwnd_raw as *mut std::ffi::c_void);
    destroy_widget_hwnd(hwnd);
}

#[cfg(windows)]
fn dispose_widget<R: tauri::Runtime>(app: &tauri::AppHandle<R>, widget: NativeWidget) {
    if widget.managed_window {
        hide_managed_widget(app, widget.hwnd);
    } else {
        destroy_hwnd(widget.hwnd);
    }
}

#[cfg(windows)]
fn hide_managed_widget<R: tauri::Runtime>(app: &tauri::AppHandle<R>, hwnd_raw: isize) {
    use tauri::Manager;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Graphics::Gdi::SetWindowRgn;
    use windows::Win32::UI::WindowsAndMessaging::{KillTimer, SetParent, ShowWindow, SW_HIDE};

    let hwnd = HWND(hwnd_raw as *mut std::ffi::c_void);
    unsafe {
        let _ = KillTimer(Some(hwnd), TIMER_ID);
        let _ = ShowWindow(hwnd, SW_HIDE);
        let _ = SetWindowRgn(hwnd, None, false);
        let _ = SetParent(hwnd, None);
    }
    if let Some(window) = app.get_webview_window("taskbar-widget") {
        let _ = window.hide();
    }
}

#[cfg(windows)]
fn destroy_widget_hwnd(hwnd: windows::Win32::Foundation::HWND) {
    use windows::Win32::UI::WindowsAndMessaging::{DestroyWindow, KillTimer, ShowWindow, SW_HIDE};

    unsafe {
        let _ = KillTimer(Some(hwnd), TIMER_ID);
        let _ = ShowWindow(hwnd, SW_HIDE);
        let _ = DestroyWindow(hwnd);
    }
}

#[cfg(windows)]
fn destroy_orphan_widgets_impl() {
    use windows::core::{w, BOOL};
    use windows::Win32::Foundation::{HWND, LPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumChildWindows, EnumWindows, FindWindowExW, FindWindowW, GetClassNameW,
    };

    unsafe extern "system" fn enum_proc(hwnd: HWND, _lparam: LPARAM) -> BOOL {
        if unsafe { is_our_widget_window(hwnd) } {
            destroy_widget_hwnd(hwnd);
        }
        true.into()
    }

    unsafe fn is_our_widget_window(hwnd: HWND) -> bool {
        let mut class_name = [0_u16; 128];
        let len = unsafe { GetClassNameW(hwnd, &mut class_name) };
        if len <= 0 {
            return false;
        }
        String::from_utf16_lossy(&class_name[..len as usize]) == "FluentFlyoutRustTaskbarWidget"
    }

    unsafe {
        let _ = EnumWindows(Some(enum_proc), LPARAM(0));

        if let Ok(shell) = FindWindowW(w!("Shell_TrayWnd"), None) {
            let _ = EnumChildWindows(Some(shell), Some(enum_proc), LPARAM(0));
        }

        let mut after: Option<HWND> = None;
        while let Ok(hwnd) = FindWindowExW(None, after, w!("Shell_SecondaryTrayWnd"), None) {
            let _ = EnumChildWindows(Some(hwnd), Some(enum_proc), LPARAM(0));
            after = Some(hwnd);
        }
    }
}

#[cfg(not(windows))]
fn destroy_orphan_widgets_impl() {}

#[cfg(windows)]
unsafe extern "system" fn widget_wnd_proc(
    hwnd: windows::Win32::Foundation::HWND,
    msg: u32,
    wparam: windows::Win32::Foundation::WPARAM,
    lparam: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::LRESULT {
    use windows::Win32::Foundation::LRESULT;
    use windows::Win32::Graphics::Gdi::{InvalidateRect, UpdateWindow};
    use windows::Win32::UI::WindowsAndMessaging::{
        DefWindowProcW, WM_CLOSE, WM_DESTROY, WM_LBUTTONUP, WM_NCDESTROY, WM_PAINT, WM_TIMER,
    };

    match msg {
        WM_CLOSE => {
            let _ = unsafe { windows::Win32::UI::WindowsAndMessaging::DestroyWindow(hwnd) };
            LRESULT(0)
        }
        WM_DESTROY | WM_NCDESTROY => {
            let _ = unsafe { windows::Win32::UI::WindowsAndMessaging::KillTimer(Some(hwnd), TIMER_ID) };
            LRESULT(0)
        }
        0x003D | 0x0018 | 0x0046 | 0x0083 | 0x0281 | 0x0282 => LRESULT(0),
        WM_TIMER => {
            keep_child_widget_on_top(hwnd);
            let should_animate = draw_state()
                .lock()
                .ok()
                .is_some_and(|draw| draw.visualizer_enabled && draw.playing);
            if should_animate {
                let _ = unsafe { InvalidateRect(Some(hwnd), None, false) };
                let _ = unsafe { UpdateWindow(hwnd) };
            }
            LRESULT(0)
        }
        WM_LBUTTONUP => {
            let action = widget_action_from_lparam(lparam).unwrap_or(TaskbarWidgetAction::ShowFlyout);
            if let Ok(guard) = click_sender().lock() {
                if let Some(sender) = guard.as_ref() {
                    let _ = sender.send(action);
                }
            }
            LRESULT(0)
        }
        WM_PAINT => {
            paint_widget(hwnd);
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

#[cfg(windows)]
fn widget_action_from_lparam(lparam: windows::Win32::Foundation::LPARAM) -> Option<TaskbarWidgetAction> {
    let x = (lparam.0 as i16) as i32;
    let y = ((lparam.0 >> 16) as i16) as i32;
    let draw = draw_state().lock().ok()?.clone();
    let placement = draw.placement.clone()?;
    if !draw.controls_enabled {
        return Some(TaskbarWidgetAction::ShowFlyout);
    }

    let metrics = control_metrics(&placement, &draw);
    if let Some((button_top, control_size, button_lefts)) = metrics {
        let bottom = button_top + control_size;
        if y < button_top || y > bottom {
            return Some(TaskbarWidgetAction::ShowFlyout);
        }

        for (index, left) in button_lefts.iter().enumerate() {
            if x >= *left && x <= *left + control_size {
                return Some(match index {
                    0 => TaskbarWidgetAction::Previous,
                    1 => TaskbarWidgetAction::PlayPause,
                    _ => TaskbarWidgetAction::Next,
                });
            }
        }
    }

    Some(TaskbarWidgetAction::ShowFlyout)
}

#[cfg(windows)]
fn control_metrics(
    placement: &TaskbarWidgetPlacementDto,
    draw: &DrawState,
) -> Option<(i32, i32, [i32; 3])> {
    if !draw.controls_enabled || !draw.has_media {
        return None;
    }

    let visual_scale = (placement.height as f64 / DEFAULT_LOGICAL_HEIGHT as f64).max(0.7);
    let control_size = (32.0 * visual_scale * ORIGINAL_WIDGET_SCALE).round() as i32;
    let control_gap = 2;
    let controls_width = control_size * 3 + control_gap * 2 + 8;
    let top = placement.widget_y + ((placement.height - control_size) / 2).max(0);
    let start = if draw.controls_position == 0 {
        placement.widget_x + 4
    } else {
        placement.widget_x + placement.width - controls_width + 4
    };

    Some((
        top,
        control_size,
        [
            start,
            start + control_size + control_gap,
            start + (control_size + control_gap) * 2,
        ],
    ))
}

#[cfg(windows)]
fn controls_width(placement: &TaskbarWidgetPlacementDto, draw: &DrawState) -> i32 {
    control_metrics(placement, draw)
        .map(|(_, control_size, _)| control_size * 3 + 2 * 2 + 8)
        .unwrap_or(0)
}

#[cfg(windows)]
fn invalidate_hwnd(hwnd_raw: isize) {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Graphics::Gdi::{InvalidateRect, UpdateWindow};

    let hwnd = HWND(hwnd_raw as *mut std::ffi::c_void);
    unsafe {
        let _ = InvalidateRect(Some(hwnd), None, false);
        let _ = UpdateWindow(hwnd);
    }
}

#[cfg(windows)]
fn keep_child_widget_on_top(hwnd: windows::Win32::Foundation::HWND) {
    use windows::Win32::UI::WindowsAndMessaging::{
        SetWindowPos, HWND_TOP, SWP_ASYNCWINDOWPOS, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
        SWP_SHOWWINDOW,
    };

    unsafe {
        let _ = SetWindowPos(
            hwnd,
            Some(HWND_TOP),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_ASYNCWINDOWPOS | SWP_SHOWWINDOW,
        );
    }
}

#[cfg(windows)]
fn paint_widget(hwnd: windows::Win32::Foundation::HWND) {
    use windows::Win32::Foundation::RECT;
    use windows::Win32::Graphics::Gdi::{
        BeginPaint, CreatePen, CreateSolidBrush, DeleteObject, EndPaint, FillRect, GetStockObject,
        RoundRect, SelectObject, SetBkMode, SetTextColor, DEFAULT_GUI_FONT, DT_CENTER,
        DT_END_ELLIPSIS, DT_LEFT, DT_SINGLELINE, DT_VCENTER, HGDIOBJ, NULL_BRUSH, PAINTSTRUCT,
        PS_SOLID, TRANSPARENT,
    };
    use windows::Win32::UI::WindowsAndMessaging::GetClientRect;

    let draw = draw_state().lock().ok().map(|state| state.clone());
    let Some(draw) = draw else {
        return;
    };
    let Some(ref placement) = draw.placement else {
        return;
    };

    unsafe {
        let mut client = RECT::default();
        if GetClientRect(hwnd, &mut client).is_err() {
            return;
        }

        let mut ps = PAINTSTRUCT::default();
        let hdc = BeginPaint(hwnd, &mut ps);
        let clear = CreateSolidBrush(colorref(1, 1, 1));
        let _ = FillRect(hdc, &client, clear);
        let _ = DeleteObject(clear.into());

        let left = placement.widget_x;
        let top = placement.widget_y;
        let right = left + placement.width;
        let bottom = top + placement.height;
        let visual_scale = (placement.height as f64 / DEFAULT_LOGICAL_HEIGHT as f64).max(0.7);
        let art_size = (ART_LOGICAL_SIZE as f64 * visual_scale * ORIGINAL_WIDGET_SCALE).round() as i32;
        let controls_width = controls_width(&placement, &draw);
        let controls_on_left = controls_width > 0 && draw.controls_position == 0;
        let art_left = if controls_on_left { left + controls_width + 2 } else { left + 4 };
        let art_top = top + ((placement.height - art_size) / 2).max(0);
        let art_right = art_left + art_size;
        let art_bottom = art_top + art_size;

        let bg = CreateSolidBrush(colorref(31, 33, 31));
        let border = CreatePen(PS_SOLID, 1, colorref(82, 84, 82));
        let old_brush = SelectObject(hdc, bg.into());
        let old_pen = SelectObject(hdc, border.into());
        let _ = RoundRect(hdc, left, top, right, bottom, 6, 6);
        let _ = SelectObject(hdc, old_brush);
        let _ = SelectObject(hdc, old_pen);
        let _ = DeleteObject(bg.into());
        let _ = DeleteObject(border.into());

        let accent = CreateSolidBrush(colorref(54, 61, 55));
        let no_pen = GetStockObject(NULL_BRUSH);
        let old_brush = SelectObject(hdc, accent.into());
        let old_pen = SelectObject(hdc, no_pen);
        let _ = RoundRect(hdc, art_left, art_top, art_right, art_bottom, 5, 5);
        let _ = SelectObject(hdc, old_brush);
        let _ = SelectObject(hdc, old_pen);
        let _ = DeleteObject(accent.into());

        let font = GetStockObject(DEFAULT_GUI_FONT);
        let old_font = SelectObject(hdc, font);
        let _ = SetBkMode(hdc, TRANSPARENT);
        let _ = SetTextColor(hdc, colorref(245, 245, 245));

        let mut icon_rect = RECT {
            left: art_left,
            top: art_top,
            right: art_right,
            bottom: art_bottom,
        };
        draw_text(
            hdc,
            if draw.has_media && !draw.playing { "Ⅱ" } else { "♪" },
            &mut icon_rect,
            DT_CENTER | DT_SINGLELINE | DT_VCENTER,
        );

        let text_right = if controls_width > 0 && !controls_on_left {
            right - controls_width
        } else {
            right - 8
        };
        let text_left = art_right + INFO_MARGIN;
        let mut title_rect = RECT {
            left: text_left,
            top: top + 5,
            right: text_right,
            bottom: top + 21,
        };
        let mut artist_rect = RECT {
            left: text_left,
            top: top + 20,
            right: text_right,
            bottom: bottom - 3,
        };
        if draw.has_media {
            draw_text(
                hdc,
                &draw.title,
                &mut title_rect,
                DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS,
            );
            let _ = SetTextColor(hdc, colorref(170, 170, 170));
            draw_text(
                hdc,
                &draw.artist,
                &mut artist_rect,
                DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS,
            );
        }

        if let Some((button_top, control_size, button_lefts)) = control_metrics(&placement, &draw) {
            let _ = SetTextColor(hdc, colorref(240, 240, 240));
            let labels = ["‹", if draw.playing { "Ⅱ" } else { "▶" }, "›"];
            for (index, label) in labels.iter().enumerate() {
                let button_left = button_lefts[index];
                let mut rect = RECT {
                    left: button_left,
                    top: button_top,
                    right: button_left + control_size,
                    bottom: button_top + control_size,
                };
                draw_text(hdc, label, &mut rect, DT_CENTER | DT_SINGLELINE | DT_VCENTER);
            }
        }

        if draw.visualizer_enabled {
            if let Some(visualizer) = &draw.visualizer_rect {
                let bg = CreateSolidBrush(colorref(31, 33, 31));
                let border = CreatePen(PS_SOLID, 1, colorref(82, 84, 82));
                let old_brush = SelectObject(hdc, bg.into());
                let old_pen = SelectObject(hdc, border.into());
                let _ = RoundRect(
                    hdc,
                    visualizer.left,
                    visualizer.top,
                    visualizer.right,
                    visualizer.bottom,
                    6,
                    6,
                );
                let _ = SelectObject(hdc, old_brush);
                let _ = SelectObject(hdc, old_pen);
                let _ = DeleteObject(bg.into());
                let _ = DeleteObject(border.into());

                let bar_brush = CreateSolidBrush(colorref(245, 245, 245));
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|value| value.as_millis() as i32)
                    .unwrap_or(0);
                let max_bar_height = (visualizer.height() - 14).max(4);
                for index in 0..draw.visualizer_bar_count {
                    let phase = ((now / 120) + index * 3) % 11;
                    let dynamic = if draw.playing {
                        let peak_height = (draw.audio_peak * max_bar_height as f32) as i32;
                        4 + peak_height * (2 + ((phase - 5).abs() - 5).abs()) / 7
                    } else {
                        4
                    };
                    let bar_height = dynamic.clamp(4, max_bar_height);
                    let bar_left = visualizer.left
                        + VISUALIZER_SIDE_PADDING
                        + index * (VISUALIZER_BAR_WIDTH + VISUALIZER_BAR_GAP);
                    let bar_bottom = visualizer.top + visualizer.height() / 2 + bar_height / 2;
                    let rect = RECT {
                        left: bar_left,
                        top: bar_bottom - bar_height,
                        right: bar_left + VISUALIZER_BAR_WIDTH,
                        bottom: bar_bottom,
                    };
                    let _ = FillRect(hdc, &rect, bar_brush);
                }
                let _ = DeleteObject(bar_brush.into());
            }
        }

        let _ = SelectObject(hdc, old_font);
        let _ = SelectObject(hdc, HGDIOBJ::default());
        let _ = EndPaint(hwnd, &ps);
    }
}
#[cfg(windows)]
unsafe fn draw_text(
    hdc: windows::Win32::Graphics::Gdi::HDC,
    text: &str,
    rect: &mut windows::Win32::Foundation::RECT,
    flags: windows::Win32::Graphics::Gdi::DRAW_TEXT_FORMAT,
) {
    use windows::Win32::Graphics::Gdi::DrawTextW;
    let mut wide: Vec<u16> = text.encode_utf16().collect();
    if wide.is_empty() {
        wide.push(0);
    }
    let _ = unsafe { DrawTextW(hdc, &mut wide, rect, flags) };
}

#[cfg(windows)]
fn colorref(r: u8, g: u8, b: u8) -> windows::Win32::Foundation::COLORREF {
    windows::Win32::Foundation::COLORREF((r as u32) | ((g as u32) << 8) | ((b as u32) << 16))
}

#[cfg(windows)]
fn parse_hwnd(value: &str) -> Option<windows::Win32::Foundation::HWND> {
    let hex = value.strip_prefix("0x")?;
    let raw = isize::from_str_radix(hex, 16).ok()?;
    if raw == 0 {
        None
    } else {
        Some(windows::Win32::Foundation::HWND(
            raw as *mut std::ffi::c_void,
        ))
    }
}

