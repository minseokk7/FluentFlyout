import { choices } from '$lib/data/settings';
import type { FeaturePageModel, PageId, SettingCardModel } from '$lib/types/app';

export const navItems: Array<{ id: PageId; title: string; icon: string }> = [
  { id: 'home', title: '홈', icon: 'home' },
  { id: 'media', title: '미디어 Flyout', icon: 'media' },
  { id: 'widget', title: '작업표시줄 위젯', icon: 'widget' },
  { id: 'next', title: '다음 곡 Flyout', icon: 'next' },
  { id: 'lock', title: '토글 키 Flyout', icon: 'lock' },
  { id: 'visualizer', title: '작업표시줄 미디어 시각화', icon: 'visualizer' },
  { id: 'system', title: '시스템', icon: 'settings' }
];

export const footerNavItems: Array<{ id: PageId; title: string; icon: string }> = [
  { id: 'about', title: '정보', icon: 'info' }
];

const toggle = (key: SettingCardModel['key'], icon: string, title: string, description = '', badge = '', sectionGap = false): SettingCardModel => ({
  kind: 'toggle',
  key,
  icon,
  title,
  description,
  badge,
  sectionGap
});
const select = (key: SettingCardModel['key'], icon: string, title: string, options: string[], description = '', badge = '', sectionGap = false): SettingCardModel => ({
  kind: 'select',
  key,
  icon,
  title,
  description,
  options,
  badge,
  sectionGap
});
const number = (key: SettingCardModel['key'], icon: string, title: string, description: string, unit: string, sectionGap = false): SettingCardModel => ({
  kind: 'number',
  key,
  icon,
  title,
  description,
  unit,
  sectionGap
});
const slider = (key: SettingCardModel['key'], icon: string, title: string, description: string, min: number, max: number, badge = '', sectionGap = false): SettingCardModel => ({
  kind: 'slider',
  key,
  icon,
  title,
  description,
  min,
  max,
  badge,
  sectionGap
});
const info = (title: string, icon = 'info', badge = '', sectionGap = false): SettingCardModel => ({
  kind: 'info',
  icon,
  title,
  badge,
  sectionGap
});
const action = (icon: string, title: string, description = '', badge = '', actionId = ''): SettingCardModel => ({
  kind: 'action',
  icon,
  title,
  description,
  badge,
  actionId
});
const expander = (icon: string, title: string, description: string, children: SettingCardModel[], sectionGap = false): SettingCardModel => ({
  kind: 'expander',
  icon,
  title,
  description,
  children,
  sectionGap
});

export const dashboardCards = [
  { page: 'media' as PageId, icon: 'media', title: '미디어 Flyout', statusKey: 'mediaFlyoutEnabled' as const },
  { page: 'widget' as PageId, icon: 'widget', title: '작업표시줄 위젯', statusKey: 'taskbarWidgetEnabled' as const, badge: 'PREMIUM' },
  { page: 'next' as PageId, icon: 'next', title: '다음 곡 Flyout', statusKey: 'nextUpEnabled' as const },
  { page: 'lock' as PageId, icon: 'lock', title: '토글 키 Flyout', statusKey: 'lockKeysEnabled' as const },
  { page: 'visualizer' as PageId, icon: 'visualizer', title: '작업표시줄 미디어 시각화', statusKey: 'taskbarVisualizerEnabled' as const, badge: 'PREMIUM' },
  { page: 'system' as PageId, icon: 'settings', title: '시스템', subtitle: '설정 및 구성' }
];

export const featurePages: Record<Exclude<PageId, 'home'>, FeaturePageModel> = {
  media: {
    id: 'media',
    title: '미디어 Flyout',
    hero: {
      image: '/assets/media.png',
      description: '미디어를 재생할 때 키보드의 미디어 또는 볼륨 키를 누르면 Flyout을 표시합니다. 재생 제어 기능과 함께 미디어 정보를 표시합니다.'
    },
    sections: [{ cards: [
      toggle('mediaFlyoutEnabled', 'media', '미디어 Flyout 활성화', '미디어 키를 누를 때 Flyout 표시', '', true),
      action('open', '미디어 Flyout 지금 표시', '현재 재생 중인 미디어로 원본 WPF Flyout을 표시합니다.', '', 'show_media_flyout'),
      select('mediaFlyoutBackgroundBlur', 'image', '배경 블러', choices.background, '세련된 배경 블러 스타일 프리셋을 목록에서 선택'),
      toggle('mediaFlyoutAcrylicWindowEnabled', 'effects', '아크릴 효과', 'Flyout에 투명한 아크릴 블러 효과를 적용합니다. 이 기능을 끄면 Mica 효과로 되돌아갑니다.'),
      toggle('compactLayout', 'layout', '레이아웃 간소화', '좁은 플라이아웃 레이아웃. 반복, 셔플, 플레이어 정보를 숨김'),
      select('position', 'position', 'Flyout 위치', choices.flyoutPosition, '', '', true),
      number('duration', 'timer', 'Flyout 표시 지속 시간', '(기본값: 3000 ms)', 'ms'),
      toggle('mediaFlyoutAlwaysDisplay', 'pin', '항상 표시', '닫기 버튼을 클릭할 때까지 Flyout을 계속 표시', '', true),
      toggle('centerTitleArtist', 'text', '제목 및 아티스트 가운데 정렬', '텍스트를 왼쪽 대신 가운데로 정렬'),
      toggle('playerInfoEnabled', 'card', '미디어 플레이어 이름 표시'),
      toggle('repeatEnabled', 'repeat', '반복 버튼*', '모두 반복 또는 하나 반복 선택. 적용 시 Flyout 너비 증가'),
      toggle('shuffleEnabled', 'shuffle', '셔플 버튼*', '적용 시 Flyout 너비 증가'),
      toggle('seekbarEnabled', 'seek', '진행 바 표시*', '플레이어가 지원하는 경우 상호 작용 가능한 진행 바가 자동으로 표시'),
      info('*일부 브라우저/플레이어는 반복, 셔플 및 진행 바 제어를 지원하지 않습니다.', 'warning', '', true),
      toggle('pauseOtherSessionsEnabled', 'pause', '다른 미디어 자동 일시 정지', '새로운 미디어를 재생하거나 포커스를 변경하면 다른 활성 세션들은 자동으로 일시 정지됨'),
      toggle('mediaFlyoutVolumeKeysExcluded', 'mute', '볼륨 키 입력 무시', '볼륨 높이기/줄이기/음소거를 누를 때 플라이아웃을 표시하지 않기'),
      toggle('exclusiveTidalMode', 'music', '타이달 전용 모드 (Exclusive Tidal Mode)', '타이달 앱의 미디어 정보와 제어에 집중합니다. 비활성화 시 일반 미디어 세션을 제어합니다.', '', true)
    ] }]
  },
  widget: {
    id: 'widget',
    title: '작업표시줄 위젯',
    hero: {
      image: '/assets/widget.png',
      description: 'Windows 작업 표시줄에 내장되어 미디어에 바로 접근할 수 있는 컴팩트한 위젯입니다. 위젯을 클릭하면 미디어 Flyout을 확장하여 확인할 수 있습니다.'
    },
    sections: [{ cards: [
      info('GitHub/직접 빌드 버전에서는 프리미엄 기능이 활성화되어 있습니다.'),
      toggle('taskbarWidgetEnabled', 'widget', '작업표시줄 위젯 활성화', '작업표시줄에 컴팩트 미디어 위젯을 표시', 'PREMIUM', true),
      action('position', '작업표시줄 위젯 다시 배치', '원본 WPF 작업표시줄 위젯 호스트를 다시 시작하고 위치를 재계산합니다.', '', 'reposition_taskbar_widget'),
      info('표시 문제가 있을 경우 아래의 옵션을 설정해주세요.', 'warning'),
      select('taskbarWidgetPosition', 'position', '위젯 위치', choices.widgetPosition),
      select('taskbarWidgetSelectedMonitor', 'display', '위젯 표시 디스플레이', choices.monitors),
      expander('wrench', '위젯 여백 설정', '작업표시줄 스타일에 맞게 위젯의 여백을 조정', [
        toggle('taskbarWidgetPadding', 'widget', 'Windows 위젯 버튼 자동 여백', 'Windows 위젯을 사용하는 경우 겹치지 않도록 여백 적용'),
        number('taskbarWidgetManualPadding', 'spacing', '사용자 지정 여백', '추가 여백 (기본값: 0px)', 'px')
      ], true),
      toggle('taskbarWidgetClickable', 'open', '미디어가 재생 중일 때 클릭하여 미디어 Flyout을 표시', '작업표시줄 위젯을 클릭하면 Flyout 표시'),
      toggle('taskbarWidgetCloseableFlyout', 'swap', '두 번 클릭하여 Flyout 닫기', 'Flyout이 열려 있을 때 작업표시줄 위젯을 클릭하여 닫기', 'NEW', true),
      toggle('taskbarWidgetBackgroundBlur', 'image', '배경 블러 효과', '작업표시줄 위젯에 배경 블러 효과를 적용'),
      toggle('taskbarWidgetHideCompletely', 'hide', '재생 중인 미디어가 없을 때 완전히 숨기기', '재생 중인 미디어가 없으면 미디어 아이콘을 표시하는 대신 위젯을 숨김'),
      toggle('taskbarWidgetControlsEnabled', 'media', '위젯 미디어 컨트롤', '위젯에 미디어 컨트롤 버튼을 표시'),
      select('taskbarWidgetControlsPosition', 'position', '미디어 컨트롤 위치', choices.controlsPosition, '미디어 컨트롤 버튼을 표시할 위치를 선택', '', true),
      toggle('taskbarWidgetAnimated', 'spark', '위젯 애니메이션', '위젯에 자연스러운 애니메이션 효과 적용', '', true)
    ] }]
  },
  next: {
    id: 'next',
    title: '다음 곡 Flyout',
    hero: {
      image: '/assets/next.png',
      description: '노래/영상이 끝날 때 Flyout에 다음 재생 목록이 표시됩니다. 이 Flyout은 다음 항목이 자동으로 재생될 경우에만 나타납니다.'
    },
    sections: [{ cards: [
      toggle('nextUpEnabled', 'next', '다음 곡 Flyout 활성화 (실험적 기능)', '음악/영상이 끝날 때 다음에 재생될 항목을 표시', '', true),
      action('open', '다음 곡 Flyout 테스트 표시', '현재 미디어 정보를 사용해 원본 WPF 다음 곡 Flyout을 표시합니다.', '', 'show_next_up_flyout'),
      toggle('nextUpAcrylicWindowEnabled', 'effects', '아크릴 효과', 'Flyout에 투명한 아크릴 블러 효과를 적용합니다.'),
      number('nextUpDuration', 'timer', '다음 곡 표시 지속 시간', '(기본값: 2000 ms)', 'ms', true)
    ] }]
  },
  lock: {
    id: 'lock',
    title: '토글 키 Flyout',
    hero: {
      image: '/assets/lock.png',
      description: 'Caps Lock, Num Lock 또는 Scroll Lock을 누르면 토글 키 상태를 한눈에 확인할 수 있습니다.'
    },
    sections: [{ cards: [
      toggle('lockKeysEnabled', 'lock', '토글 키 Flyout 활성화', '지정한 토글 키의 활성화/비활성화를 시각적으로 표시', '', true),
      action('open', '토글 키 Flyout 테스트 표시', '현재 Caps Lock 상태로 원본 WPF 토글 키 Flyout을 표시합니다.', '', 'show_lock_keys_flyout'),
      toggle('lockKeysAcrylicWindowEnabled', 'effects', '아크릴 효과', 'Flyout에 투명한 아크릴 블러 효과를 적용합니다.'),
      number('lockKeysDuration', 'timer', '토글 키 표시 지속 시간', '(기본값: 2000 ms)', 'ms'),
      toggle('lockKeysBoldUi', 'bold', '기호 및 글꼴 굵게 표시', 'Flyout 기호 및 글꼴을 굵게 표시'),
      select('lockKeysMonitorPreference', 'display', '표시 대상 디스플레이', choices.lockMonitor, 'Flyout을 표시할 디스플레이를 선택합니다.', 'NEW'),
      toggle('lockKeysInsertEnabled', 'info', 'Insert 키 팝업 활성화', 'Insert/Overwrite 키를 누를 때 Flyout 표시 여부 설정', '', true)
    ] }]
  },
  visualizer: {
    id: 'visualizer',
    title: '작업표시줄 미디어 시각화',
    hero: {
      image: '/assets/visualizer.png',
      description: '작업표시줄에 실시간 오디오 시각화를 표시합니다. 최신 시스템에서 성능에 최소한의 영향만 주도록 설계되었습니다.'
    },
    sections: [{ cards: [
      info('GitHub/직접 빌드 버전에서는 프리미엄 기능이 활성화되어 있습니다.'),
      toggle('taskbarVisualizerEnabled', 'visualizer', '작업표시줄 시각화 활성화 (실험적 기능)', '이 기능은 작업표시줄 위젯이 켜져 있을 때만 작동', 'PREMIUM', true),
      select('taskbarVisualizerPosition', 'position', '작업표시줄 위젯에서 시각화를 표시할 위치', choices.visualizerPosition),
      slider('taskbarVisualizerBarCount', 'bars', '바 개수', '시각화에 표시되는 바의 개수 설정', 1, 20),
      toggle('taskbarVisualizerCenteredBars', 'center', '중앙 대칭형 바', '바가 중앙에서 대칭을 이루며 확장'),
      toggle('taskbarVisualizerBaseline', 'line', '기준선 표시', '오디오 출력이 0일 때 얇은 선 표시', '', true),
      slider('taskbarVisualizerAudioSensitivity', 'wave', '오디오 민감도', '바가 움직이기 시작하는 최소 오디오 크기', 1, 3),
      slider('taskbarVisualizerAudioPeakLevel', 'peak', '오디오 피크 임계값', '바가 최대 높이에 도달하기 위한 오디오 크기 설정', 1, 3, 'NEW'),
      action('speaker', '출력 장치', 'FluentFlyout이 시각화를 출력할 오디오 장치를 변경합니다.', '', 'open_audio_settings'),
      toggle('taskbarVisualizerClickable', 'open', 'Click to open Settings', 'Clicking the visualizer will open its settings page', 'NEW', true)
    ] }]
  },
  system: {
    id: 'system',
    title: '시스템',
    sections: [
      { title: '외형 & 동작', cards: [
        select('flyoutSelectedMonitor', 'display', 'Flyout 표시 디스플레이', choices.monitors),
        slider('acrylicBlurOpacity', 'effects', '아크릴 블러 불투명도', 'Flyout의 아크릴 블러 효과 투명도를 조정합니다 (기본값: 175)', 0, 255, 'PREMIUM'),
        toggle('useAlbumArtAsAccentColor', 'accent', 'Use Album Art as Accent Color', 'Some UI elements will use the album art color instead of the system theme', 'NEW'),
        select('appLanguageIndex', 'language', '앱 언어', choices.language, '사용하는 언어가 목록에 없거나 번역이 불완전한 경우 기본적으로 영어로 표시됩니다.'),
        select('appTheme', 'palette', '앱 테마', choices.theme),
        toggle('startup', 'startup', '시작 시 실행', 'Windows 로그인 시 트레이로 최소화하여 시작'),
        toggle('disableIfFullscreen', 'fullscreen', 'DirectX 전체 화면 프로그램이 감지되면 Flyout 비활성화', 'Flyout이 나타날 때 DirectX 게임이 최소화되는 것을 방지')
      ]},
      { title: '트레이 아이콘', cards: [
        select('niconLeftClick', 'tray', '트레이 아이콘 왼쪽 클릭 동작', choices.trayLeftClick, '트레이 아이콘을 마우스 좌클릭했을 때 실행될 동작 선택'),
        toggle('niconSymbol', 'tray', 'Windows 11 스타일 트레이 아이콘', 'Windows 11 기본 테마 스타일의 트레이 아이콘을 사용'),
        toggle('niconHide', 'hide', '트레이 아이콘 완전히 숨기기', '프로그램을 두 번 실행하여 설정 표시', 'PREMIUM')
      ]},
      { title: '백업 & 복원', cards: [
        action('backup', '설정을 백업하거나 복원', '', 'NEW', 'backup_settings')
      ]},
      { title: '업데이트', cards: [
        toggle('showUpdateNotifications', 'bell', '새 업데이트가 있을 시 알림 표시'),
        action('advanced', 'Advanced Settings', '', 'NEW', 'open_advanced_settings')
      ]}
    ]
  },
  about: {
    id: 'about',
    title: '정보',
    sections: [
      { title: '제작에 도움을 주신 분들을 소개합니다', cards: [
        action('code', '개발', '', '', 'open_repository'),
        action('language', '번역', '', '', 'open_weblate')
      ]},
      { title: '프리미엄 기능', cards: [
        info('GitHub/직접 빌드 버전에서는 프리미엄 기능이 활성화되어 있습니다.'),
        action('heart', '후원하기', '', '', 'open_sponsor'),
        action('coffee', '커피 한 잔 사주기', '', '', 'open_coffee')
      ]},
      { title: '오픈 소스 라이선스', cards: [
        info('CommunityToolkit.Mvvm  8.4.0-preview3'),
        info('Dubya.WindowsMediaController  2.5.5'),
        info('MicaWPF  6.3.2'),
        info('Microsoft.Toolkit.Uwp.Notifications  7.1.3'),
        info('NAudio  2.2.1'),
        info('NLog  6.0.6'),
        info('System.Drawing.Common  10.0.0'),
        info('WPF-UI  4.2.0'),
        info('WPF-UI.Tray  4.2.0')
      ]}
    ]
  }
};
