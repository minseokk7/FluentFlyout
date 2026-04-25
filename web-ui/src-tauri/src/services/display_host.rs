use std::{
    env,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{Mutex, OnceLock},
    thread,
    time::Duration,
};

use crate::error::{AppError, AppResult};
use windows::{
    core::HSTRING,
    Win32::{
        Foundation::CloseHandle,
        System::Threading::{OpenEventW, SetEvent, EVENT_MODIFY_STATE},
    },
};

static DISPLAY_HOST_PROCESS: OnceLock<Mutex<Option<Child>>> = OnceLock::new();

fn process_slot() -> &'static Mutex<Option<Child>> {
    DISPLAY_HOST_PROCESS.get_or_init(|| Mutex::new(None))
}

pub fn start() -> AppResult<()> {
    if let Ok(mut guard) = process_slot().lock() {
        if guard.as_mut().is_some_and(|child| child.try_wait().ok().flatten().is_none()) {
            return Ok(());
        }
    }

    let root = find_repo_root().ok_or_else(|| {
        AppError::new(
            "display_host.repo_not_found",
            "표시 전용 WPF 호스트 프로젝트 경로를 찾을 수 없습니다.",
        )
    })?;

    let exe = find_display_host_exe(&root).or_else(|| {
        build_display_host(&root).ok()?;
        find_display_host_exe(&root)
    });

    let exe = exe.ok_or_else(|| {
        AppError::new(
            "display_host.exe_not_found",
            "표시 전용 WPF 호스트 실행 파일을 찾을 수 없습니다.",
        )
    })?;

    let child = Command::new(&exe)
        .current_dir(exe.parent().unwrap_or(&root))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| {
            AppError::with_detail(
                "display_host.start_failed",
                "표시 전용 WPF 호스트 실행에 실패했습니다.",
                error,
            )
        })?;

    if let Ok(mut guard) = process_slot().lock() {
        *guard = Some(child);
    }

    Ok(())
}

pub fn show_media_flyout() -> AppResult<()> {
    start()?;
    signal_event("Local\\FluentFlyout_DisplayHost_ShowMediaFlyout")
}

pub fn show_next_up_flyout() -> AppResult<()> {
    start()?;
    signal_event("Local\\FluentFlyout_DisplayHost_ShowNextUpFlyout")
}

pub fn show_lock_keys_flyout() -> AppResult<()> {
    start()?;
    signal_event("Local\\FluentFlyout_DisplayHost_ShowLockKeysFlyout")
}

pub fn reload_settings() -> AppResult<()> {
    start()?;
    signal_event("Local\\FluentFlyout_DisplayHost_ReloadSettings")
}

pub fn stop_if_owned() {
    if let Ok(mut guard) = process_slot().lock() {
        if let Some(mut child) = guard.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

fn signal_event(name: &str) -> AppResult<()> {
    let name = HSTRING::from(name);
    let mut last_error = None;
    let mut handle = None;

    // 호스트 프로세스를 막 시작한 직후에는 named event 생성보다 신호가 먼저 갈 수 있다.
    // 짧게 재시도해서 UI 명령이 간헐적으로 누락되는 문제를 막는다.
    for _ in 0..20 {
        match unsafe { OpenEventW(EVENT_MODIFY_STATE, false, &name) } {
            Ok(opened) => {
                handle = Some(opened);
                break;
            }
            Err(error) => {
                last_error = Some(error);
                thread::sleep(Duration::from_millis(50));
            }
        }
    }

    let handle = handle.ok_or_else(|| {
        AppError::with_detail(
            "display_host.event_open_failed",
            "WPF 표시 호스트에 명령을 보낼 수 없습니다.",
            last_error
                .map(|error| error.to_string())
                .unwrap_or_else(|| "named event를 찾을 수 없습니다.".to_string()),
        )
    })?;

    unsafe { SetEvent(handle) }.map_err(|error| {
        let _ = unsafe { CloseHandle(handle) };
        AppError::with_detail(
            "display_host.event_signal_failed",
            "WPF 표시 호스트 명령 실행에 실패했습니다.",
            error,
        )
    })?;
    unsafe { CloseHandle(handle) }.map_err(|error| {
        AppError::with_detail(
            "display_host.event_close_failed",
            "WPF 표시 호스트 명령 핸들을 닫지 못했습니다.",
            error,
        )
    })?;
    Ok(())
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
                .join("FluentFlyoutDisplayHost")
                .join("FluentFlyoutDisplayHost.csproj")
                .exists()
        })
}

fn find_display_host_exe(root: &Path) -> Option<PathBuf> {
    let base = root.join("FluentFlyoutDisplayHost").join("bin");
    let candidates = [
        base.join("x64")
            .join("Debug")
            .join("net10.0-windows10.0.22000.0")
            .join("FluentFlyoutDisplayHost.exe"),
        base.join("Debug")
            .join("net10.0-windows10.0.22000.0")
            .join("FluentFlyoutDisplayHost.exe"),
        base.join("x64")
            .join("Release")
            .join("net10.0-windows10.0.22000.0")
            .join("FluentFlyoutDisplayHost.exe"),
        base.join("Release")
            .join("net10.0-windows10.0.22000.0")
            .join("FluentFlyoutDisplayHost.exe"),
    ];

    candidates.into_iter().find(|path| path.exists())
}

fn build_display_host(root: &Path) -> AppResult<()> {
    let status = Command::new("dotnet")
        .args([
            "build",
            "FluentFlyoutDisplayHost\\FluentFlyoutDisplayHost.csproj",
            "-c",
            "Debug",
            "-p:Platform=x64",
        ])
        .current_dir(root)
        .status()
        .map_err(|error| {
            AppError::with_detail(
                "display_host.build_start_failed",
                "표시 전용 WPF 호스트 빌드를 시작할 수 없습니다.",
                error,
            )
        })?;

    if status.success() {
        Ok(())
    } else {
        Err(AppError::with_detail(
            "display_host.build_failed",
            "표시 전용 WPF 호스트 빌드에 실패했습니다.",
            status,
        ))
    }
}
