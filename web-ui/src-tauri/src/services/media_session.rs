use std::sync::OnceLock;

use base64::{engine::general_purpose, Engine as _};
use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSession,
    GlobalSystemMediaTransportControlsSessionManager,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus,
};
use windows::Storage::Streams::DataReader;

use crate::{
    error::{AppError, AppResult},
    models::MediaSessionDto,
};

static MANAGER: OnceLock<GlobalSystemMediaTransportControlsSessionManager> = OnceLock::new();
const MAX_ALBUM_ART_BYTES: u64 = 2 * 1024 * 1024;

pub async fn get_media_session() -> AppResult<MediaSessionDto> {
    let Some(session) = current_session()? else {
        return Ok(empty_session());
    };

    let mut title = String::new();
    let mut artist = String::new();
    let mut album_art_data_url = None;
    if let Ok(properties_operation) = session.TryGetMediaPropertiesAsync() {
        if let Ok(properties) = properties_operation.get() {
            title = properties.Title().unwrap_or_default().to_string();
            artist = properties.Artist().unwrap_or_default().to_string();
            album_art_data_url = album_art_data_url_from_properties(&properties);
        }
    }

    let mut playback_status = "stopped".to_string();
    let mut can_play = false;
    let mut can_pause = false;
    let mut can_previous = false;
    let mut can_next = false;

    if let Ok(info) = session.GetPlaybackInfo() {
        playback_status = match info
            .PlaybackStatus()
            .unwrap_or(GlobalSystemMediaTransportControlsSessionPlaybackStatus::Closed)
        {
            GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing => "playing",
            GlobalSystemMediaTransportControlsSessionPlaybackStatus::Paused => "paused",
            _ => "stopped",
        }
        .to_string();

        if let Ok(controls) = info.Controls() {
            can_play = controls.IsPlayEnabled().unwrap_or(false);
            can_pause = controls.IsPauseEnabled().unwrap_or(false);
            can_previous = controls.IsPreviousEnabled().unwrap_or(false);
            can_next = controls.IsNextEnabled().unwrap_or(false);
        }
    }

    if title.trim().is_empty() {
        title = "No media playing".to_string();
        playback_status = "stopped".to_string();
    }

    Ok(MediaSessionDto {
        title,
        artist,
        album_art_data_url,
        playback_status,
        can_play,
        can_pause,
        can_previous,
        can_next,
    })
}

pub async fn control(action: String) -> AppResult<()> {
    let Some(session) = current_session()? else {
        return Err(AppError::new("media.no_session", "제어할 수 있는 미디어 세션이 없습니다."));
    };

    let result = match action.as_str() {
        "playPause" => session.TryTogglePlayPauseAsync(),
        "play" => session.TryPlayAsync(),
        "pause" => session.TryPauseAsync(),
        "next" => session.TrySkipNextAsync(),
        "previous" => session.TrySkipPreviousAsync(),
        _ => return Err(AppError::new("media.invalid_action", "지원하지 않는 미디어 제어 명령입니다.")),
    };

    let accepted = result
        .map_err(|error| AppError::with_detail("media.control.request", "미디어 제어 요청을 보낼 수 없습니다.", error))?
        .get()
        .map_err(|error| AppError::with_detail("media.control.execute", "미디어 제어 명령 실행에 실패했습니다.", error))?;

    if accepted {
        Ok(())
    } else {
        Err(AppError::new(
            "media.control.rejected",
            "현재 플레이어가 이 미디어 제어 명령을 거부했습니다.",
        ))
    }
}

fn current_session() -> AppResult<Option<GlobalSystemMediaTransportControlsSession>> {
    let manager = media_manager()?;
    match manager.GetCurrentSession() {
        Ok(session) => Ok(Some(session)),
        Err(_) => Ok(None),
    }
}

fn media_manager() -> AppResult<&'static GlobalSystemMediaTransportControlsSessionManager> {
    if let Some(manager) = MANAGER.get() {
        return Ok(manager);
    }

    let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
        .map_err(|error| AppError::with_detail("media.manager.request", "Windows 미디어 세션 관리자 요청에 실패했습니다.", error))?
        .get()
        .map_err(|error| AppError::with_detail("media.manager.create", "Windows 미디어 세션 관리자를 초기화할 수 없습니다.", error))?;

    let _ = MANAGER.set(manager);
    MANAGER
        .get()
        .ok_or_else(|| AppError::new("media.manager.cache", "Windows 미디어 세션 관리자를 사용할 수 없습니다."))
}

fn album_art_data_url_from_properties(
    properties: &windows::Media::Control::GlobalSystemMediaTransportControlsSessionMediaProperties,
) -> Option<String> {
    let thumbnail = properties.Thumbnail().ok()?;
    let stream = thumbnail.OpenReadAsync().ok()?.get().ok()?;
    let size = stream.Size().ok()?.min(MAX_ALBUM_ART_BYTES);
    if size == 0 {
        return None;
    }

    let input = stream.GetInputStreamAt(0).ok()?;
    let reader = DataReader::CreateDataReader(&input).ok()?;
    let loaded = reader.LoadAsync(size as u32).ok()?.get().ok()?;
    if loaded == 0 {
        return None;
    }

    let length = reader.UnconsumedBufferLength().ok()? as usize;
    let mut bytes = vec![0_u8; length];
    reader.ReadBytes(&mut bytes).ok()?;
    let content_type = stream
        .ContentType()
        .ok()
        .map(|value| value.to_string())
        .filter(|value| value.starts_with("image/"))
        .unwrap_or_else(|| "image/jpeg".to_string());
    Some(format!(
        "data:{content_type};base64,{}",
        general_purpose::STANDARD.encode(bytes)
    ))
}

fn empty_session() -> MediaSessionDto {
    MediaSessionDto {
        title: "No media playing".to_string(),
        artist: String::new(),
        album_art_data_url: None,
        playback_status: "stopped".to_string(),
        can_play: false,
        can_pause: false,
        can_previous: false,
        can_next: false,
    }
}
