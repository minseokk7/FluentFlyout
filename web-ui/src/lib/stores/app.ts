import { derived, writable } from 'svelte/store';
import { choices, defaultSettings } from '$lib/data/settings';
import { backend } from '$lib/services/backend';
import type {
  MediaSessionDto,
  MonitorDto,
  PageId,
  SettingsDto,
  SettingsPatch,
  TaskbarWidgetPlacementDto
} from '$lib/types/app';

const fallbackErrorMessage = '요청한 동작을 실행할 수 없습니다.';

export const currentPage = writable<PageId>('home');
export const settingsStore = writable<SettingsDto>(structuredClone(defaultSettings));
export const mediaStore = writable<MediaSessionDto>({
  title: 'No media playing',
  artist: '',
  albumArtDataUrl: null,
  playbackStatus: 'stopped',
  canPlay: true,
  canPause: true,
  canPrevious: true,
  canNext: true
});
export const monitorStore = writable<MonitorDto[]>([
  { id: 0, name: choices.monitors[0], left: 0, top: 0, width: 1920, height: 1080, isPrimary: true }
]);
export const taskbarWidgetStore = writable<TaskbarWidgetPlacementDto | null>(null);
export const appErrorStore = writable<string | null>(null);

export const dashboardStatus = derived(settingsStore, ($settings) => ({
  media: $settings.mediaFlyoutEnabled,
  widget: $settings.taskbarWidgetEnabled,
  next: $settings.nextUpEnabled,
  lock: $settings.lockKeysEnabled,
  visualizer: $settings.taskbarVisualizerEnabled
}));

export async function initializeApp(): Promise<void> {
  try {
    const [settings, media, monitors] = await Promise.all([
      backend.getSettings(),
      backend.getMediaSession(),
      backend.listMonitors()
    ]);
    settingsStore.set(settings);
    mediaStore.set(media);
    monitorStore.set(monitors);
    appErrorStore.set(null);
  } catch (error) {
    appErrorStore.set(error instanceof Error ? error.message : fallbackErrorMessage);
  }
}

export async function updateSetting(patch: SettingsPatch): Promise<void> {
  settingsStore.update((current) => ({ ...current, ...patch }));
  try {
    const updated = await backend.updateSettings(patch);
    settingsStore.set(updated);
    appErrorStore.set(null);
  } catch (error) {
    appErrorStore.set(error instanceof Error ? error.message : fallbackErrorMessage);
    const restored = await backend.getSettings();
    settingsStore.set(restored);
  }
}

export async function refreshTaskbarWidgetPlacement(): Promise<void> {
  try {
    taskbarWidgetStore.set(await backend.repositionTaskbarWidget());
  } catch (error) {
    appErrorStore.set(error instanceof Error ? error.message : fallbackErrorMessage);
  }
}

export async function runAppAction(action: string): Promise<void> {
  try {
    await backend.appAction(action);
    appErrorStore.set(null);
  } catch (error) {
    appErrorStore.set(error instanceof Error ? error.message : fallbackErrorMessage);
  }
}
