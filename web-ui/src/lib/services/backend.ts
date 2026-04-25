import type {
  AppError,
  FlyoutShowOptions,
  MediaAction,
  MediaSessionDto,
  MonitorDto,
  SettingsDto,
  SettingsPatch,
  TaskbarWidgetPlacementDto
} from '$lib/types/app';
import { defaultSettings } from '$lib/data/settings';

const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri) return browserFallback<T>(command, args);

  try {
    const api = await import('@tauri-apps/api/core');
    return await api.invoke<T>(command, args);
  } catch (error) {
    const appError = error as Partial<AppError> & { messageKo?: string; message?: string };
    const message =
      appError?.message_ko ??
      appError?.messageKo ??
      appError?.message ??
      `명령 실행 중 오류가 발생했습니다: ${command}`;
    throw new Error(appError?.detail ? `${message} (${appError.detail})` : message);
  }
}

function browserFallback<T>(command: string, args?: Record<string, unknown>): T {
  if (command === 'get_settings') return structuredClone(defaultSettings) as T;
  if (command === 'update_settings') return { ...defaultSettings, ...(args?.patch as SettingsPatch) } as T;
  if (command === 'get_media_session') {
    return {
      title: 'No media playing',
      artist: '',
      albumArtDataUrl: null,
      playbackStatus: 'stopped',
      canPlay: true,
      canPause: true,
      canPrevious: true,
      canNext: true
    } as T;
  }
  if (command === 'list_monitors') {
    return [{ id: 0, name: '* 1 (Generic PnP Monitor)', left: 0, top: 0, width: 1920, height: 1080, isPrimary: true }] as T;
  }
  if (command === 'reposition_taskbar_widget') {
    return {
      monitorId: 0,
      taskbarHwnd: 'browser',
      x: 0,
      y: 0,
      width: 120,
      height: 40,
      containerX: 0,
      containerY: 0,
      containerWidth: 1920,
      containerHeight: 40,
      widgetX: 0,
      widgetY: 0,
      source: 'browser-preview'
    } as T;
  }
  return undefined as T;
}

export const backend = {
  getSettings: () => call<SettingsDto>('get_settings'),
  updateSettings: (patch: SettingsPatch) => call<SettingsDto>('update_settings', { patch }),
  getMediaSession: () => call<MediaSessionDto>('get_media_session'),
  mediaControl: (action: MediaAction) => call<void>('media_control', { action }),
  showMediaFlyout: (options: FlyoutShowOptions) => call<void>('show_media_flyout', { options }),
  setTaskbarWidgetEnabled: (enabled: boolean) => call<void>('set_taskbar_widget_enabled', { enabled }),
  repositionTaskbarWidget: () => call<TaskbarWidgetPlacementDto>('reposition_taskbar_widget'),
  listMonitors: () => call<MonitorDto[]>('list_monitors'),
  appAction: (action: string) => call<void>('app_action', { action })
};
