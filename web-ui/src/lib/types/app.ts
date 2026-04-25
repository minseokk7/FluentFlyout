export type PageId = 'home' | 'media' | 'widget' | 'next' | 'lock' | 'visualizer' | 'system' | 'about';

export interface AppError {
  code: string;
  message_ko: string;
  detail?: string | null;
}

export interface SettingsDto {
  mediaFlyoutEnabled: boolean;
  mediaFlyoutBackgroundBlur: number;
  mediaFlyoutAcrylicWindowEnabled: boolean;
  compactLayout: boolean;
  position: number;
  duration: number;
  mediaFlyoutAlwaysDisplay: boolean;
  centerTitleArtist: boolean;
  playerInfoEnabled: boolean;
  repeatEnabled: boolean;
  shuffleEnabled: boolean;
  seekbarEnabled: boolean;
  pauseOtherSessionsEnabled: boolean;
  mediaFlyoutVolumeKeysExcluded: boolean;
  exclusiveTidalMode: boolean;
  nextUpEnabled: boolean;
  nextUpAcrylicWindowEnabled: boolean;
  nextUpDuration: number;
  lockKeysEnabled: boolean;
  lockKeysAcrylicWindowEnabled: boolean;
  lockKeysDuration: number;
  lockKeysBoldUi: boolean;
  lockKeysMonitorPreference: number;
  lockKeysInsertEnabled: boolean;
  taskbarWidgetEnabled: boolean;
  taskbarWidgetPosition: number;
  taskbarWidgetSelectedMonitor: number;
  taskbarWidgetPadding: boolean;
  taskbarWidgetManualPadding: number;
  taskbarWidgetClickable: boolean;
  taskbarWidgetCloseableFlyout: boolean;
  taskbarWidgetBackgroundBlur: boolean;
  taskbarWidgetHideCompletely: boolean;
  taskbarWidgetControlsEnabled: boolean;
  taskbarWidgetControlsPosition: number;
  taskbarWidgetAnimated: boolean;
  taskbarVisualizerEnabled: boolean;
  taskbarVisualizerPosition: number;
  taskbarVisualizerBarCount: number;
  taskbarVisualizerCenteredBars: boolean;
  taskbarVisualizerBaseline: boolean;
  taskbarVisualizerAudioSensitivity: number;
  taskbarVisualizerAudioPeakLevel: number;
  taskbarVisualizerClickable: boolean;
  flyoutSelectedMonitor: number;
  acrylicBlurOpacity: number;
  useAlbumArtAsAccentColor: boolean;
  appLanguageIndex: number;
  appTheme: number;
  startup: boolean;
  disableIfFullscreen: boolean;
  niconLeftClick: number;
  niconSymbol: boolean;
  niconHide: boolean;
  showUpdateNotifications: boolean;
  flyoutAnimationSpeed: number;
  flyoutAnimationEasingStyle: number;
  legacyTaskbarWidthEnabled: boolean;
}

export type SettingsPatch = Partial<SettingsDto>;

export interface MediaSessionDto {
  title: string;
  artist: string;
  albumArtDataUrl?: string | null;
  playbackStatus: 'playing' | 'paused' | 'stopped';
  canPlay: boolean;
  canPause: boolean;
  canPrevious: boolean;
  canNext: boolean;
}

export type MediaAction = 'playPause' | 'play' | 'pause' | 'next' | 'previous';

export interface FlyoutShowOptions {
  toggleMode: boolean;
  forceShow: boolean;
}

export interface MonitorDto {
  id: number;
  name: string;
  left: number;
  top: number;
  width: number;
  height: number;
  isPrimary: boolean;
}

export interface TaskbarWidgetPlacementDto {
  monitorId: number;
  taskbarHwnd: string;
  x: number;
  y: number;
  width: number;
  height: number;
  dpiScale: number;
  logicalWidth: number;
  logicalHeight: number;
  containerX: number;
  containerY: number;
  containerWidth: number;
  containerHeight: number;
  widgetX: number;
  widgetY: number;
  source: string;
}

export type ControlKind = 'toggle' | 'select' | 'number' | 'slider' | 'info' | 'action' | 'expander';

export interface SettingCardModel {
  kind: ControlKind;
  key?: keyof SettingsDto;
  icon: string;
  title: string;
  description?: string;
  badge?: string;
  options?: string[];
  unit?: string;
  min?: number;
  max?: number;
  sectionGap?: boolean;
  actionId?: string;
  children?: SettingCardModel[];
}

export interface FeaturePageModel {
  id: PageId;
  title: string;
  hero?: {
    image: string;
    description: string;
  };
  sections: Array<{
    title?: string;
    cards: SettingCardModel[];
  }>;
}
