use crate::{
    error::{AppError, AppResult},
    models::SettingsDto,
};

#[cfg(windows)]
pub fn output_peak(settings: &SettingsDto) -> AppResult<f32> {
    use windows::Win32::Media::Audio::{
        eConsole, eRender, IMMDeviceEnumerator, MMDeviceEnumerator,
    };
    use windows::Win32::Media::Audio::Endpoints::IAudioMeterInformation;
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED,
    };

    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)
            .map_err(|error| {
                AppError::with_detail(
                    "audio.enumerator",
                    "오디오 출력 장치 정보를 초기화할 수 없습니다.",
                    error,
                )
            })?;
        let device = enumerator.GetDefaultAudioEndpoint(eRender, eConsole).map_err(|error| {
            AppError::with_detail(
                "audio.default_endpoint",
                "기본 오디오 출력 장치를 찾을 수 없습니다.",
                error,
            )
        })?;
        let meter: IAudioMeterInformation = device.Activate(CLSCTX_ALL, None).map_err(|error| {
            AppError::with_detail(
                "audio.meter",
                "오디오 출력 레벨을 읽을 수 없습니다.",
                error,
            )
        })?;
        let peak = meter.GetPeakValue().map_err(|error| {
            AppError::with_detail("audio.peak", "오디오 피크 값을 읽을 수 없습니다.", error)
        })?;

        let sensitivity = match settings.taskbar_visualizer_audio_sensitivity {
            1 => 0.85,
            3 => 1.35,
            _ => 1.0,
        };
        let peak_level = match settings.taskbar_visualizer_audio_peak_level {
            1 => 0.75,
            3 => 1.25,
            _ => 1.0,
        };
        Ok((peak * sensitivity * peak_level).clamp(0.0, 1.0))
    }
}

#[cfg(not(windows))]
pub fn output_peak(_settings: &SettingsDto) -> AppResult<f32> {
    Ok(0.0)
}
