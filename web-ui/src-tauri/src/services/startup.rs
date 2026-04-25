use std::process::Command;

use tauri::{AppHandle, Manager};

use crate::{
    error::{AppError, AppResult},
    models::SettingsDto,
};

const RUN_KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
const APP_VALUE_NAME: &str = "FluentFlyout";

pub fn apply_startup_setting(app: &AppHandle, settings: &SettingsDto) -> AppResult<()> {
    if settings.startup {
        enable_startup(app)
    } else {
        disable_startup()
    }
}

fn enable_startup(app: &AppHandle) -> AppResult<()> {
    let exe_path = std::env::current_exe()
        .or_else(|_| app.path().resolve("FluentFlyout.exe", tauri::path::BaseDirectory::Resource))
        .map_err(|error| AppError::with_detail("startup.path", "시작 프로그램에 등록할 실행 파일 경로를 확인할 수 없습니다.", error))?;
    let value = format!("\"{}\" --minimized", exe_path.display());

    let status = Command::new("reg")
        .args(["add", RUN_KEY, "/v", APP_VALUE_NAME, "/t", "REG_SZ", "/d", &value, "/f"])
        .status()
        .map_err(|error| AppError::with_detail("startup.reg.launch", "Windows 시작 프로그램 등록 명령을 실행할 수 없습니다.", error))?;

    if status.success() {
        Ok(())
    } else {
        Err(AppError::with_detail("startup.reg.add", "Windows 시작 프로그램 등록에 실패했습니다.", format!("종료 코드: {:?}", status.code())))
    }
}

fn disable_startup() -> AppResult<()> {
    let status = Command::new("reg")
        .args(["delete", RUN_KEY, "/v", APP_VALUE_NAME, "/f"])
        .status()
        .map_err(|error| AppError::with_detail("startup.reg.launch", "Windows 시작 프로그램 해제 명령을 실행할 수 없습니다.", error))?;

    if status.success() || status.code() == Some(1) {
        Ok(())
    } else {
        Err(AppError::with_detail("startup.reg.delete", "Windows 시작 프로그램 해제에 실패했습니다.", format!("종료 코드: {:?}", status.code())))
    }
}