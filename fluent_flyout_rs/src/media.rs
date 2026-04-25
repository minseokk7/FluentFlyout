use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSession,
    GlobalSystemMediaTransportControlsSessionManager,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus,
};
use windows::Media::MediaPlaybackAutoRepeatMode;
use windows::Storage::Streams::DataReader;

#[derive(Clone, Debug)]
pub struct MediaSnapshot {
    pub session_key: String,
    pub title: String,
    pub artist: String,
    pub player_label: String,
    pub current_duration: String,
    pub max_duration: String,
    pub position_seconds: f32,
    pub max_seek_seconds: f32,
    pub repeat_label: String,
    pub repeat_active: bool,
    pub shuffle_active: bool,
    pub can_toggle_repeat: bool,
    pub can_toggle_shuffle: bool,
    pub supports_seekbar: bool,
    pub is_playing: bool,
    pub album_art_bytes: Option<Vec<u8>>,
    pub has_album_art: bool,
}

pub fn load_current_media_snapshot(exclusive_tidal_mode: bool) -> Result<Option<MediaSnapshot>, String> {
    let manager = request_manager()?;

    let session = select_session(&manager, exclusive_tidal_mode)?;
    let Some(session) = session else {
        return Ok(None);
    };

    let source_app_id = session
        .SourceAppUserModelId()
        .map(|value| value.to_string())
        .unwrap_or_default();

    let mut title = String::from("No media playing");
    let mut artist = String::new();
    let mut album_art_bytes = None;

    if let Ok(props_op) = session.TryGetMediaPropertiesAsync() {
        if let Ok(props) = props_op.get() {
            title = props.Title().map(|value| value.to_string()).unwrap_or(title);
            artist = props.Artist().map(|value| value.to_string()).unwrap_or_default();
            album_art_bytes = load_thumbnail_bytes(&props);
        }
    }

    let playback_info = session
        .GetPlaybackInfo()
        .map_err(|error| format!("GetPlaybackInfo failed: {error}"))?;

    let playback_status = playback_info
        .PlaybackStatus()
        .unwrap_or(GlobalSystemMediaTransportControlsSessionPlaybackStatus::Closed);
    let controls = playback_info.Controls().ok();
    let can_toggle_repeat = controls
        .as_ref()
        .and_then(|value| value.IsRepeatEnabled().ok())
        .unwrap_or(false);
    let can_toggle_shuffle = controls
        .as_ref()
        .and_then(|value| value.IsShuffleEnabled().ok())
        .unwrap_or(false);
    let repeat_mode = playback_info
        .AutoRepeatMode()
        .ok()
        .and_then(|value| value.Value().ok())
        .unwrap_or(MediaPlaybackAutoRepeatMode::None);
    let shuffle_active = playback_info
        .IsShuffleActive()
        .ok()
        .and_then(|value| value.Value().ok())
        .unwrap_or(false);

    let timeline = session.GetTimelineProperties().ok();
    let max_seek_ticks = timeline
        .as_ref()
        .and_then(|value| value.MaxSeekTime().ok())
        .map(|value| value.Duration)
        .unwrap_or_default();
    let position_ticks = timeline
        .as_ref()
        .and_then(|value| value.Position().ok())
        .map(|value| value.Duration)
        .unwrap_or_default();

    Ok(Some(MediaSnapshot {
        session_key: format!(
            "{source_app_id}|{title}|{artist}|{}",
            if playback_status == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing {
                "playing"
            } else {
                "idle"
            }
        ),
        title,
        artist,
        player_label: sanitize_player_label(&source_app_id),
        current_duration: format_ticks(position_ticks),
        max_duration: format_ticks(max_seek_ticks),
        position_seconds: ticks_to_seconds(position_ticks),
        max_seek_seconds: ticks_to_seconds(max_seek_ticks),
        repeat_label: match repeat_mode {
            MediaPlaybackAutoRepeatMode::Track => String::from("1"),
            _ => String::from("R"),
        },
        repeat_active: repeat_mode != MediaPlaybackAutoRepeatMode::None,
        shuffle_active,
        can_toggle_repeat,
        can_toggle_shuffle,
        supports_seekbar: max_seek_ticks >= 10_000_000,
        is_playing: playback_status == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing,
        has_album_art: album_art_bytes.is_some(),
        album_art_bytes,
    }))
}

pub fn toggle_play_pause(exclusive_tidal_mode: bool) -> Result<bool, String> {
    with_selected_session(exclusive_tidal_mode, |session| {
        session
            .TryTogglePlayPauseAsync()
            .map(|_| true)
            .map_err(|error| format!("TryTogglePlayPauseAsync failed: {error}"))
    })
}

pub fn skip_next(exclusive_tidal_mode: bool) -> Result<bool, String> {
    with_selected_session(exclusive_tidal_mode, |session| {
        session
            .TrySkipNextAsync()
            .map(|_| true)
            .map_err(|error| format!("TrySkipNextAsync failed: {error}"))
    })
}

pub fn skip_previous(exclusive_tidal_mode: bool) -> Result<bool, String> {
    with_selected_session(exclusive_tidal_mode, |session| {
        session
            .TrySkipPreviousAsync()
            .map(|_| true)
            .map_err(|error| format!("TrySkipPreviousAsync failed: {error}"))
    })
}

pub fn seek_to_seconds(exclusive_tidal_mode: bool, seconds: f32) -> Result<bool, String> {
    let ticks = (seconds.max(0.0) * 10_000_000.0).round() as i64;

    with_selected_session(exclusive_tidal_mode, |session| {
        session
            .TryChangePlaybackPositionAsync(ticks)
            .map(|_| true)
            .map_err(|error| format!("TryChangePlaybackPositionAsync failed: {error}"))
    })
}

pub fn toggle_shuffle(exclusive_tidal_mode: bool) -> Result<bool, String> {
    with_selected_session(exclusive_tidal_mode, |session| {
        let playback_info = session
            .GetPlaybackInfo()
            .map_err(|error| format!("GetPlaybackInfo failed: {error}"))?;
        let shuffle_active = playback_info
            .IsShuffleActive()
            .map_err(|error| format!("IsShuffleActive failed: {error}"))?
            .Value()
            .map_err(|error| format!("IsShuffleActive.Value failed: {error}"))?;

        session
            .TryChangeShuffleActiveAsync(!shuffle_active)
            .map(|_| true)
            .map_err(|error| format!("TryChangeShuffleActiveAsync failed: {error}"))
    })
}

pub fn cycle_repeat_mode(exclusive_tidal_mode: bool) -> Result<bool, String> {
    with_selected_session(exclusive_tidal_mode, |session| {
        let playback_info = session
            .GetPlaybackInfo()
            .map_err(|error| format!("GetPlaybackInfo failed: {error}"))?;
        let current_mode = playback_info
            .AutoRepeatMode()
            .map_err(|error| format!("AutoRepeatMode failed: {error}"))?
            .Value()
            .map_err(|error| format!("AutoRepeatMode.Value failed: {error}"))?;

        let next_mode = match current_mode {
            MediaPlaybackAutoRepeatMode::None => MediaPlaybackAutoRepeatMode::List,
            MediaPlaybackAutoRepeatMode::List => MediaPlaybackAutoRepeatMode::Track,
            MediaPlaybackAutoRepeatMode::Track => MediaPlaybackAutoRepeatMode::None,
            _ => MediaPlaybackAutoRepeatMode::None,
        };

        session
            .TryChangeAutoRepeatModeAsync(next_mode)
            .map(|_| true)
            .map_err(|error| format!("TryChangeAutoRepeatModeAsync failed: {error}"))
    })
}

fn select_session(
    manager: &GlobalSystemMediaTransportControlsSessionManager,
    exclusive_tidal_mode: bool,
) -> Result<Option<GlobalSystemMediaTransportControlsSession>, String> {
    if exclusive_tidal_mode {
        let sessions = manager
            .GetSessions()
            .map_err(|error| format!("GetSessions failed: {error}"))?;

        for session in sessions {
            let source = session
                .SourceAppUserModelId()
                .map(|value| value.to_string())
                .unwrap_or_default();

            if source.contains("TIDAL") {
                return Ok(Some(session));
            }
        }

        return Ok(None);
    }

    manager
        .GetCurrentSession()
        .map(Some)
        .map_err(|error| format!("GetCurrentSession failed: {error}"))
}

fn request_manager() -> Result<GlobalSystemMediaTransportControlsSessionManager, String> {
    GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
        .map_err(|error| format!("RequestAsync failed: {error}"))?
        .get()
        .map_err(|error| format!("RequestAsync.get failed: {error}"))
}

fn with_selected_session<T>(
    exclusive_tidal_mode: bool,
    f: impl FnOnce(&GlobalSystemMediaTransportControlsSession) -> Result<T, String>,
) -> Result<T, String> {
    let manager = request_manager()?;
    let session = select_session(&manager, exclusive_tidal_mode)?;
    let Some(session) = session else {
        return Err(String::from("No active media session"));
    };

    f(&session)
}

fn sanitize_player_label(source_app_id: &str) -> String {
    if source_app_id.is_empty() {
        return String::from("Unknown");
    }

    if source_app_id.contains("TIDAL") {
        return String::from("TIDAL");
    }

    if source_app_id.contains("Spotify") {
        return String::from("Spotify");
    }

    let mut segments = source_app_id
        .split(['.', '_', ' ', '!'])
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .filter(|segment| {
            !matches!(
                segment.to_ascii_lowercase().as_str(),
                "com" | "exe" | "github" | "application"
            )
        });

    if let Some(first) = segments.next() {
        return first.to_string();
    }

    source_app_id.to_string()
}

fn format_ticks(ticks: i64) -> String {
    if ticks <= 0 {
        return String::from("-:--");
    }

    let total_seconds = ticks / 10_000_000;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    if hours > 0 {
        format!("{hours:02}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes}:{seconds:02}")
    }
}

fn ticks_to_seconds(ticks: i64) -> f32 {
    if ticks <= 0 {
        0.0
    } else {
        ticks as f32 / 10_000_000.0
    }
}

fn load_thumbnail_bytes(
    props: &windows::Media::Control::GlobalSystemMediaTransportControlsSessionMediaProperties,
) -> Option<Vec<u8>> {
    let thumbnail = props.Thumbnail().ok()?;
    let stream = thumbnail.OpenReadAsync().ok()?.get().ok()?;
    let size = stream.Size().ok()? as usize;
    if size == 0 {
        return None;
    }

    let mut encoded = vec![0u8; size];
    let reader = DataReader::CreateDataReader(&stream).ok()?;
    reader.LoadAsync(size as u32).ok()?.get().ok()?;
    reader.ReadBytes(&mut encoded).ok()?;

    Some(encoded)
}
