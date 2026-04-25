(function () {
  const assets = {
    icon: "../FluentFlyoutWPF/Resources/FluentFlyout2.ico",
    media: "../FluentFlyoutWPF/Resources/FluentFlyoutDemo3.1.png",
    next: "../FluentFlyoutWPF/Resources/FluentFlyoutDemo4.1.png",
    lock: "../FluentFlyoutWPF/Resources/FluentFlyoutDemo5.1.png",
    widget: "../FluentFlyoutWPF/Resources/FluentFlyoutDemo9.1.png",
    visualizer: "../FluentFlyoutWPF/Resources/FluentFlyoutTaskbar-NoTitle-Zoomed.png",
    update: "../FluentFlyoutWPF/Resources/logo-change.png",
    icons: "../fluent_flyout_rs/ui/icons/"
  };

  const state = {
    mediaFlyoutEnabled: true,
    mediaFlyoutBackgroundBlur: 2,
    mediaFlyoutAcrylicWindowEnabled: true,
    compactLayout: false,
    position: 1,
    durationText: "3000",
    mediaFlyoutAlwaysDisplay: false,
    centerTitleArtist: false,
    playerInfoEnabled: true,
    repeatEnabled: true,
    shuffleEnabled: true,
    seekbarEnabled: true,
    pauseOtherSessionsEnabled: false,
    mediaFlyoutVolumeKeysExcluded: false,
    exclusiveTidalMode: false,
    taskbarWidgetEnabled: true,
    taskbarWidgetPosition: 1,
    taskbarWidgetSelectedMonitor: 0,
    taskbarWidgetPadding: false,
    taskbarWidgetManualPaddingText: "0",
    taskbarWidgetClickable: true,
    taskbarWidgetCloseableFlyout: true,
    taskbarWidgetBackgroundBlur: true,
    taskbarWidgetHideCompletely: false,
    taskbarWidgetControlsEnabled: true,
    taskbarWidgetControlsPosition: 1,
    taskbarWidgetAnimated: true,
    nextUpEnabled: true,
    nextUpAcrylicWindowEnabled: true,
    nextUpDurationText: "2000",
    lockKeysEnabled: true,
    lockKeysAcrylicWindowEnabled: true,
    lockKeysDurationText: "2000",
    lockKeysBoldUi: false,
    lockKeysMonitorPreference: 0,
    lockKeysInsertEnabled: true,
    taskbarVisualizerEnabled: true,
    taskbarVisualizerPosition: 0,
    taskbarVisualizerBarCount: 12,
    taskbarVisualizerCenteredBars: true,
    taskbarVisualizerBaseline: true,
    taskbarVisualizerAudioSensitivity: 2,
    taskbarVisualizerAudioPeakLevel: 2,
    taskbarVisualizerClickable: true,
    flyoutSelectedMonitor: 0,
    acrylicBlurOpacity: 175,
    useAlbumArtAsAccentColor: true,
    appLanguageIndex: 0,
    appTheme: 0,
    startup: true,
    disableIfFullscreen: true,
    niconLeftClick: 0,
    niconSymbol: true,
    niconHide: false,
    showUpdateNotifications: true,
    flyoutAnimationSpeed: 2,
    flyoutAnimationEasingStyle: 2,
    legacyTaskbarWidthEnabled: false
  };

  const choices = {
    monitors: ["* 1 (Generic PnP Monitor)", "2 (External Display)"],
    background: ["없음", "스타일 1", "스타일 2", "스타일 3"],
    flyoutPosition: ["좌측 하단", "중앙 하단", "우측 하단", "좌측 상단", "중앙 상단", "우측 상단"],
    widgetPosition: ["좌측 하단", "중앙 하단", "우측 하단"],
    visualizerPosition: ["좌측 하단", "우측 하단"],
    controlsPosition: ["왼쪽", "오른쪽"],
    theme: ["Windows 기본 설정", "Light", "Dark"],
    language: ["System", "English"],
    trayLeftClick: ["설정 열기", "미디어 Flyout 표시"],
    lockMonitor: ["기본값 (Flyout 표시 대상 디스플레이)", "포커스된 창이 있는 디스플레이", "커서가 있는 디스플레이"],
    animationSpeed: ["끄기", "0.5x", "1x", "1.5x", "2x", "3x"],
    easing: ["Linear", "Sine", "Quadratic", "Cubic"]
  };

  const navItems = [
    { id: "home", title: "홈", icon: "home.svg" },
    { id: "media", title: "미디어 Flyout", icon: "media.svg" },
    { id: "widget", title: "작업표시줄 위젯", icon: "widget.svg" },
    { id: "next", title: "다음 곡 Flyout", icon: "nextup.svg" },
    { id: "lock", title: "토글 키 Flyout", icon: "lock.svg" },
    { id: "visualizer", title: "작업표시줄 미디어 시각화", icon: "visualizer.svg" },
    { id: "system", title: "시스템", icon: "settings.svg" }
  ];

  const footerItems = [
    { id: "about", title: "정보", icon: "info.svg" }
  ];

  const descriptions = {
    media: "미디어를 재생할 때 키보드의 미디어 또는 볼륨 키를 눌러 Flyout을 표시합니다. 재생 제어 기능과 함께 미디어 정보를 표시합니다.",
    widget: "Windows 작업 표시줄에 내장되어 미디어에 바로 접근할 수 있는 컴팩트한 위젯입니다. 위젯을 클릭하면 미디어 Flyout을 확장하여 확인할 수 있습니다.",
    next: "노래/영상이 끝날 때 Flyout에 다음 재생 목록이 표시됩니다. 다음 항목이 자동으로 재생될 경우에만 나타납니다.",
    lock: "토글 키 상태를 한눈에 확인하세요. Caps Lock, Num Lock 또는 Scroll Lock을 누르면 나타납니다.",
    visualizer: "작업표시줄에 실시간 오디오 시각화를 표시합니다. 최신 시스템에서 성능에 최소한의 영향만 주도록 설계되었습니다."
  };

  const pages = {
    media: {
      title: "미디어 Flyout",
      hero: { image: assets.media, text: descriptions.media },
      cards: [
        toggle("mediaFlyoutEnabled", "media-card.svg", "미디어 Flyout 활성화", "미디어 키를 누를 때 Flyout 표시", "spacer-bottom"),
        select("mediaFlyoutBackgroundBlur", "media-card.svg", "배경 블러", "세련된 배경 블러 스타일 프리셋을 목록에서 선택", choices.background),
        toggle("mediaFlyoutAcrylicWindowEnabled", "settings-card.svg", "아크릴 효과", "Flyout에 투명한 아크릴 블러 효과를 적용합니다. 이 기능을 끄면 Mica 효과로 되돌아갑니다."),
        toggle("compactLayout", "media.svg", "레이아웃 간소화", "덜 슬림하고 덜 거슬리는 레이아웃\n반복, 셔플, 플레이어 정보를 숨김"),
        select("position", "settings-card.svg", "Flyout 위치", "", choices.flyoutPosition, "spacer-bottom"),
        input("durationText", "settings-card.svg", "Flyout 표시 지속 시간", "(기본값: 3000 ms)", "ms"),
        toggle("mediaFlyoutAlwaysDisplay", "settings-card.svg", "항상 표시", "닫기 버튼을 클릭할 때까지 Flyout을 계속 표시", "spacer-bottom"),
        toggle("centerTitleArtist", "media-card.svg", "제목 및 아티스트 가운데 정렬", "텍스트를 왼쪽 대신 가운데 정렬"),
        toggle("playerInfoEnabled", "media-card.svg", "미디어 플레이어 이름 표시", "", "spacer-bottom"),
        toggle("repeatEnabled", "media-card.svg", "반복 버튼*", "모두 반복 또는 하나 반복 선택, 적용 시 Flyout 너비 증가"),
        toggle("shuffleEnabled", "media-card.svg", "셔플 버튼*", "적용 시 Flyout 너비 증가"),
        toggle("seekbarEnabled", "media-card.svg", "진행 바 표시*", "플레이어가 지원하는 경우 상호 작용 가능한 진행 바가 자동으로 표시"),
        info("*일부 브라우저/플레이어는 반복, 셔플 및 진행 바 제어를 지원하지 않습니다.", "warning", "spacer-bottom"),
        toggle("pauseOtherSessionsEnabled", "media-card.svg", "다른 미디어 자동 일시 정지", "새로운 미디어를 재생하거나 포커스를 변경하면 다른 활성 세션들은 자동으로 일시 정지됨"),
        toggle("mediaFlyoutVolumeKeysExcluded", "media-card.svg", "볼륨 키 입력 무시", "볼륨 높이기/줄이기/음소거를 누를 때 플라이아웃을 표시하지 않기"),
        toggle("exclusiveTidalMode", "media-card.svg", "타이달 전용 모드 (Exclusive Tidal Mode)", "타이달 앱의 미디어 정보와 제어에 집중합니다. 비활성화 시 일반 미디어 세션을 제어합니다.", "large-bottom")
      ]
    },
    widget: {
      title: "작업표시줄 위젯",
      hero: { image: assets.widget, text: descriptions.widget },
      cards: [
        info("GitHub/직접 빌드 버전에서는 프리미엄 기능이 활성화되어 있습니다", "info"),
        toggle("taskbarWidgetEnabled", "widget-card.svg", "작업표시줄 위젯 활성화", "작업표시줄에 컴팩트 미디어 위젯을 표시", "spacer-bottom", "PREMIUM"),
        info("표시 문제가 있을 경우 아래의 옵션을 설정해주세요.", "warning"),
        select("taskbarWidgetPosition", "settings-card.svg", "위젯 위치", "", choices.widgetPosition),
        select("taskbarWidgetSelectedMonitor", "settings-card.svg", "위젯 표시 디스플레이", "", choices.monitors),
        expander("위젯 여백 설정", "작업표시줄 스타일에 맞게 위젯의 여백을 조정", "settings.svg", [
          toggle("taskbarWidgetPadding", "settings-card.svg", "Windows 위젯 버튼 자동 여백", "Windows 위젯을 사용하는 경우 켜세요"),
          input("taskbarWidgetManualPaddingText", "settings-card.svg", "사용자 지정 여백", "추가 여백 (기본값: 0px)", "px")
        ], "spacer-bottom"),
        toggle("taskbarWidgetClickable", "widget.svg", "미디어가 재생 중일 때 클릭하여 미디어 Flyout을 표시", "작업표시줄 위젯을 클릭하면 Flyout 표시"),
        toggle("taskbarWidgetCloseableFlyout", "widget.svg", "두 번 클릭하여 Flyout 닫기", "Flyout이 열려 있을 때 작업표시줄 위젯을 클릭하여 닫기", "spacer-bottom", "NEW", true),
        toggle("taskbarWidgetBackgroundBlur", "widget-card.svg", "배경 블러 효과", "작업표시줄 위젯에 배경 블러 효과를 적용"),
        toggle("taskbarWidgetHideCompletely", "widget-card.svg", "재생 중인 미디어가 없을 때 완전히 숨김", "재생 중인 미디어가 없을 때 미디어 아이콘 대신 위젯을 숨김", "spacer-bottom"),
        toggle("taskbarWidgetControlsEnabled", "media-card.svg", "위젯 미디어 컨트롤", "위젯에 미디어 컨트롤 버튼을 표시"),
        select("taskbarWidgetControlsPosition", "settings-card.svg", "미디어 컨트롤 위치", "미디어 컨트롤 버튼을 표시할 위치를 선택", choices.controlsPosition, "spacer-bottom"),
        toggle("taskbarWidgetAnimated", "settings-card.svg", "위젯 애니메이션", "위젯에 자연스러운 애니메이션 효과 적용", "large-bottom")
      ]
    },
    next: {
      title: "다음 곡 Flyout",
      hero: { image: assets.next, text: descriptions.next },
      cards: [
        toggle("nextUpEnabled", "nextup-card.svg", "다음 곡 Flyout 활성화 (실험적 기능)", "음악/영상이 끝날 때 다음에 재생될 항목을 표시", "spacer-bottom"),
        toggle("nextUpAcrylicWindowEnabled", "settings-card.svg", "아크릴 효과", "Flyout에 투명한 아크릴 블러 효과를 적용합니다. 이 기능을 끄면 Mica 효과로 되돌아갑니다."),
        input("nextUpDurationText", "settings-card.svg", "다음 곡 표시 지속 시간", "(기본값: 2000 ms)", "ms", "large-bottom")
      ]
    },
    lock: {
      title: "토글 키 Flyout",
      hero: { image: assets.lock, text: descriptions.lock },
      cards: [
        toggle("lockKeysEnabled", "lock-card.svg", "토글 키 Flyout 활성화", "지정한 토글 키의 활성화/비활성화를 시각적으로 표시", "spacer-bottom"),
        toggle("lockKeysAcrylicWindowEnabled", "settings-card.svg", "아크릴 효과", "Flyout에 투명한 아크릴 블러 효과를 적용합니다. 이 기능을 끄면 Mica 효과로 되돌아갑니다."),
        input("lockKeysDurationText", "settings-card.svg", "토글 키 표시 지속 시간", "(기본값: 2000 ms)", "ms"),
        toggle("lockKeysBoldUi", "settings-card.svg", "기호 및 글꼴 굵게 표시", "Flyout 기호 및 글꼴을 굵게 표시"),
        select("lockKeysMonitorPreference", "settings-card.svg", "표시 대상 디스플레이", "Flyout을 표시할 디스플레이를 선택합니다.", choices.lockMonitor, "", "NEW", true),
        toggle("lockKeysInsertEnabled", "info.svg", "Insert 키 팝업 활성화", "Insert/Overwrite 키를 누를 때 Flyout 표시 여부 설정", "large-bottom")
      ]
    },
    visualizer: {
      title: "작업표시줄 미디어 시각화",
      hero: { image: assets.visualizer, text: descriptions.visualizer },
      cards: [
        info("GitHub/직접 빌드 버전에서는 프리미엄 기능이 활성화되어 있습니다", "info"),
        toggle("taskbarVisualizerEnabled", "visualizer-card.svg", "작업표시줄 시각화 활성화 (실험적 기능)", "실험중인 기능: 이 기능은 작업표시줄 위젯이 켜져 있을 때만 작동", "spacer-bottom", "PREMIUM"),
        select("taskbarVisualizerPosition", "settings-card.svg", "작업표시줄 위젯에서 시각화를 표시할 위치", "", choices.visualizerPosition),
        slider("taskbarVisualizerBarCount", "visualizer.svg", "바 개수", "시각화에 표시되는 바의 개수 설정", 1, 20),
        toggle("taskbarVisualizerCenteredBars", "visualizer.svg", "중앙 대칭형 바", "바가 바닥에서 솟아오르는 대신 중앙에서 대칭을 이루며 확장"),
        toggle("taskbarVisualizerBaseline", "visualizer.svg", "기준선 표시", "오디오 출력이 0일 때 얇은 선 표시", "spacer-bottom"),
        slider("taskbarVisualizerAudioSensitivity", "visualizer.svg", "오디오 민감도", "바가 움직이기 시작하는 최소 오디오 크기", 1, 3),
        slider("taskbarVisualizerAudioPeakLevel", "visualizer.svg", "오디오 피크 임계값", "바가 최대 높이에 도달하기 위한 오디오 크기 설정", 1, 3, "NEW", true),
        action("settings-card.svg", "출력 장치", "FluentFlyout이 시각화를 출력할 오디오 장치를 변경합니다."),
        toggle("taskbarVisualizerClickable", "visualizer-card.svg", "Click to open Settings", "Clicking the visualizer will open its settings page", "large-bottom", "NEW", true)
      ]
    },
    system: {
      title: "시스템",
      sections: [
        {
          title: "외형 & 동작",
          cards: [
            select("flyoutSelectedMonitor", "settings-card.svg", "Flyout 표시 디스플레이", "", choices.monitors),
            slider("acrylicBlurOpacity", "settings-card.svg", "아크릴 블러 불투명도", "Flyout의 아크릴 블러 효과 투명도를 조정합니다 (기본값 : 175)", 0, 255, "PREMIUM"),
            toggle("useAlbumArtAsAccentColor", "settings-card.svg", "Use Album Art as Accent Color", "Some UI elements will use the album art color instead of the system theme", "", "NEW", true),
            select("appLanguageIndex", "settings-card.svg", "앱 언어", "사용하시는 언어가 목록에 없거나 번역이 불완전한 경우, 기본적으로 영어로 표시됩니다.\n번역에 기여하고 싶으시면 Weblate에서 참여하실 수 있습니다.", choices.language),
            select("appTheme", "settings-card.svg", "앱 테마", "", choices.theme),
            toggle("startup", "settings-card.svg", "시작 시 실행", "Windows 로그인 시 트레이로 최소화하여 시작\n설정이 작동하지 않을 경우 시작 프로그램 설정을 확인하세요."),
            toggle("disableIfFullscreen", "settings-card.svg", "DirectX 전체 화면 프로그램이 감지되면 Flyout 비활성화", "Flyout이 나타날 때 DirectX 게임이 최소화되는 것을 방지")
          ]
        },
        {
          title: "트레이 아이콘",
          cards: [
            select("niconLeftClick", "settings-card.svg", "트레이 아이콘 왼쪽 클릭 동작", "트레이 아이콘을 마우스 좌클릭했을 때 실행될 동작 선택", choices.trayLeftClick),
            toggle("niconSymbol", "settings-card.svg", "Windows 11 스타일 트레이 아이콘", "Windows 11 기본 테마 스타일의 트레이 아이콘을 사용"),
            toggle("niconHide", "settings-card.svg", "트레이 아이콘 완전히 숨기기", "프로그램을 두 번 실행하여 설정 표시", "", "PREMIUM")
          ]
        },
        {
          title: "백업 & 복원",
          cards: [
            action("settings-card.svg", "설정을 백업하거나 복원", "", "NEW", true, [
              { text: "내보내기" },
              { text: "가져오기" }
            ])
          ]
        },
        {
          title: "업데이트",
          cards: [
            toggle("showUpdateNotifications", "settings-card.svg", "새 업데이트가 있을 시 알림 표시", "")
          ]
        },
        {
          cards: [
            action("settings.svg", "Advanced Settings", "", "NEW", true, null, "advanced")
          ]
        }
      ]
    },
    advanced: {
      title: "Advanced Settings",
      sections: [
        {
          title: "System",
          cards: [
            expander("Animation Settings", "Tweak the animations to your liking", "settings.svg", [
              select("flyoutAnimationSpeed", "settings-card.svg", "Animation Duration Scale", "Faster or slower flyout animation speed (default: 1x)", choices.animationSpeed),
              select("flyoutAnimationEasingStyle", "settings-card.svg", "Animation Easing Style", "(default: Quadratic)", choices.easing),
              info("For the native Windows 11 experience, the settings should be kept at default.", "info")
            ]),
            toggle("legacyTaskbarWidthEnabled", "settings-card.svg", "Legacy Taskbar Width System", "Use the legacy Taskbar width system for Taskbar Widgets", "large-bottom", "PREMIUM")
          ]
        }
      ]
    },
    about: {
      title: "About",
      about: true
    }
  };

  const licenses = [
    ["CommunityToolkit.Mvvm", "8.4.0-preview3", "MIT"],
    ["Dubya.WindowsMediaController", "2.5.5", "MIT"],
    ["MicaWPF", "6.3.2", "MIT"],
    ["Microsoft.Toolkit.Uwp.Notifications", "7.1.3", "MIT"],
    ["NAudio", "2.2.1", "MIT"],
    ["NLog", "6.0.6", "BSD-3-Clause"],
    ["System.Drawing.Common", "10.0.0", "MIT"],
    ["WPF-UI", "4.2.0", "MIT"],
    ["WPF-UI.Tray", "4.2.0", "MIT"]
  ];

  function toggle(key, icon, title, subtitle, spacing, badge, success) {
    return { type: "toggle", key, icon, title, subtitle, spacing, badge, success };
  }

  function select(key, icon, title, subtitle, options, spacing, badge, success) {
    return { type: "select", key, icon, title, subtitle, options, spacing, badge, success };
  }

  function input(key, icon, title, subtitle, unit, spacing, badge, success) {
    return { type: "input", key, icon, title, subtitle, unit, spacing, badge, success };
  }

  function slider(key, icon, title, subtitle, min, max, badge, success) {
    return { type: "slider", key, icon, title, subtitle, min, max, badge, success };
  }

  function info(text, tone, spacing) {
    return { type: "info", text, tone, spacing };
  }

  function action(icon, title, subtitle, badge, success, buttons, target) {
    return { type: "action", icon, title, subtitle, badge, success, buttons, target };
  }

  function expander(title, subtitle, icon, children, spacing) {
    return { type: "expander", title, subtitle, icon, children, spacing };
  }

  function iconPath(icon) {
    return assets.icons + icon;
  }

  function currentPageId() {
    const page = location.hash.replace("#", "");
    return pages[page] ? page : "home";
  }

  function el(tag, className, text) {
    const node = document.createElement(tag);
    if (className) node.className = className;
    if (text !== undefined) node.textContent = text;
    return node;
  }

  function renderNav() {
    document.getElementById("primary-nav").replaceChildren(...navItems.map(renderNavButton));
    document.getElementById("footer-nav").replaceChildren(...footerItems.map(renderNavButton));
  }

  function renderNavButton(item) {
    const button = el("button", "nav-item" + (item.id === currentPageId() ? " active" : ""));
    button.type = "button";
    button.addEventListener("click", () => navigate(item.id));
    const icon = el("img", "nav-icon");
    icon.src = iconPath(item.icon);
    icon.alt = "";
    button.append(icon, el("span", "nav-label", item.title));
    return button;
  }

  function navigate(id) {
    location.hash = id;
  }

  function render() {
    renderNav();
    const id = currentPageId();
    const pageRoot = document.getElementById("page-root");
    if (id === "home") {
      pageRoot.replaceChildren(renderHome());
    } else if (pages[id].about) {
      pageRoot.replaceChildren(renderAbout());
    } else {
      pageRoot.replaceChildren(renderPage(pages[id]));
    }
    pageRoot.focus({ preventScroll: true });
  }

  function renderHome() {
    const page = el("section", "page");

    const titleRow = el("div", "home-title-row");
    titleRow.append(el("h1", "", "FluentFlyout 설정"), el("span", "version", "debug version"));

    const homeTitle = el("h1", "home-section-title", "홈");
    const dashboardTitle = el("h2", "section-title", "대시보드");
    const grid = el("div", "dashboard-grid");
    [
      ["media", "media-card.svg", "미디어 Flyout", boolText(state.mediaFlyoutEnabled)],
      ["widget", "widget-card.svg", "작업표시줄 위젯", boolText(state.taskbarWidgetEnabled), true],
      ["next", "nextup-card.svg", "다음 곡 Flyout", boolText(state.nextUpEnabled)],
      ["lock", "lock-card.svg", "토글 키 Flyout", boolText(state.lockKeysEnabled)],
      ["visualizer", "visualizer-card.svg", "작업표시줄 미디어 시각화", boolText(state.taskbarVisualizerEnabled), true],
      ["system", "settings-card.svg", "시스템", "설정 및 구성"]
    ].forEach(([target, icon, title, subtitle, premium]) => {
      grid.append(renderActionCard({ type: "action", icon, title, subtitle, badge: premium ? "PREMIUM" : "", target }));
    });

    const linkRow = el("div", "link-row home-link-row");
    [
      ["store.svg", "Microsoft Store"],
      ["folder.svg", "로그 보기"],
      ["bug.svg", "오류 보고"]
    ].forEach(([icon, label]) => linkRow.append(renderLink(label, icon)));

    const premiumTitle = el("h2", "section-title home-section-title", "프리미엄 기능");
    const premium = el("div", "card premium-card");
    premium.append(
      el("p", "", "GitHub/직접 빌드 버전에서는 프리미엄 기능이 활성화되어 있습니다"),
      renderPerks([
        "작업표시줄 위젯 및 시각화",
        "아크릴 블러 불투명도",
        "트레이 아이콘 숨기기",
        "향후 모든 프리미엄 기능"
      ]),
      renderInfoBar("GitHub/직접 빌드 버전에서는 프리미엄 기능이 활성화되어 있습니다", "info")
    );

    const sponsorRow = el("div", "link-row");
    sponsorRow.append(renderLink("후원하기", "info.svg"), renderLink("커피 한 잔 사주기", "info.svg"));

    page.append(titleRow, homeTitle, dashboardTitle, grid, linkRow, premiumTitle, premium, sponsorRow, renderFooterNote());
    return page;
  }

  function renderPage(pageData) {
    const page = el("section", "page");
    page.append(el("h1", "", pageData.title));
    if (pageData.hero) page.append(renderHero(pageData.hero));
    if (pageData.cards) page.append(renderCardStack(pageData.cards));
    if (pageData.sections) {
      pageData.sections.forEach((section) => {
        if (section.title) page.append(el("h2", "section-title", section.title));
        page.append(renderCardStack(section.cards || []));
      });
    }
    return page;
  }

  function renderAbout() {
    const page = el("section", "page");
    page.append(el("h1", "", "제작에 도움을 주신 분들을 소개합니다"));
    page.append(el("p", "plain-text", "FluentFlyout의 개발과 성공에 기여해 주신 모든 멋진 분들께 진심으로 감사의 인사를 전합니다!"));
    page.append(expander("개발", "", "info.svg", [{ type: "text", text: "unchihugo and contributors" }]));
    page.append(expander("번역", "", "info.svg", [{ type: "text", text: "Community translators on Weblate" }]));

    page.append(el("h2", "section-title home-section-title", "프리미엄 기능"));
    page.append(renderInfoBar("GitHub/직접 빌드 버전에서는 프리미엄 기능이 활성화되어 있습니다", "info"));
    page.append(el("p", "plain-text", "후원은 FluentFlyout의 지속적인 개발과 업데이트에 큰 힘이 됩니다. 후원 즉시 모든 프리미엄 커스텀 기능을 영구적으로 이용하실 수 있습니다.\n\n참고: 본 프로젝트는 오픈 소스로 운영되며, GitHub의 수동 설치 파일을 통해 해당 기능들을 무료로 이용하실 수 있습니다. 다만 이 경우 설치와 업데이트를 직접 수동으로 진행해야 합니다."));
    const sponsorRow = el("div", "link-row");
    sponsorRow.append(renderLink("후원하기", "info.svg"), renderLink("커피 한 잔 사주기", "info.svg"));
    page.append(sponsorRow);

    page.append(el("h2", "section-title home-section-title", "오픈 소스 라이선스"));
    page.append(el("p", "plain-text", "FluentFlyout는 다음의 오픈 소스 패키지들을 사용하여 제작되었습니다:"));
    const list = el("div", "license-list");
    licenses.forEach(([name, version, license]) => {
      const card = el("div", "card license-card");
      const heading = el("div", "license-heading");
      heading.append(el("span", "license-name", name), el("span", "license-version", version));
      card.append(heading, el("div", "license-type", license + " license"));
      list.append(card);
    });
    page.append(list, renderFooterNote());
    return page;
  }

  function renderHero(hero) {
    const node = el("div", "hero");
    const image = el("img");
    image.src = hero.image;
    image.alt = "";
    node.append(image, el("p", "", hero.text));
    return node;
  }

  function renderCardStack(cards) {
    const stack = el("div", "cards-stack");
    cards.forEach((card) => stack.append(renderCard(card)));
    return stack;
  }

  function renderCard(card) {
    if (card.type === "info") return renderInfoBar(card.text, card.tone, card.spacing);
    if (card.type === "expander") return renderExpander(card);
    if (card.type === "action") return renderActionCard(card);
    if (card.type === "text") return el("p", "plain-text", card.text);

    const node = el("div", "card" + spacingClass(card.spacing));
    const row = el("div", "card-row");
    const icon = el("img", "card-icon");
    icon.src = iconPath(card.icon);
    icon.alt = "";

    const body = el("div", "card-body");
    const titleLine = el("div", "card-title-line");
    titleLine.append(el("div", "card-title", card.title));
    if (card.badge) titleLine.append(renderBadge(card.badge, card.success));
    body.append(titleLine);
    if (card.subtitle) body.append(el("div", "card-subtitle", card.subtitle));

    row.append(icon, body, renderControl(card));
    node.append(row);
    return node;
  }

  function renderActionCard(card) {
    const node = el("button", "card action-card" + spacingClass(card.spacing));
    node.type = "button";
    if (card.target) node.addEventListener("click", () => navigate(card.target));
    const row = el("div", "card-row");
    const icon = el("img", "card-icon");
    icon.src = iconPath(card.icon);
    icon.alt = "";
    const body = el("div", "card-body");
    const titleLine = el("div", "card-title-line");
    titleLine.append(el("div", "card-title", card.title));
    if (card.badge) titleLine.append(renderBadge(card.badge, card.success));
    body.append(titleLine);
    if (card.subtitle) body.append(el("div", "card-subtitle", card.subtitle));
    let control;
    if (card.buttons) {
      control = el("div", "control-pair");
      card.buttons.forEach((item) => control.append(el("button", "button", item.text)));
    } else {
      control = el("span", "chevron", ">");
    }
    row.append(icon, body, control);
    node.append(row);
    return node;
  }

  function renderExpander(card) {
    const details = el("details", "expander" + spacingClass(card.spacing));
    const summary = el("summary");
    const row = el("div", "card-row");
    const icon = el("img", "card-icon");
    icon.src = iconPath(card.icon);
    icon.alt = "";
    const body = el("div", "card-body");
    body.append(el("div", "card-title", card.title));
    if (card.subtitle) body.append(el("div", "card-subtitle", card.subtitle));
    row.append(icon, body, el("span", "chevron", ">"));
    summary.append(row);
    details.append(summary);
    const content = el("div", "nested-stack");
    card.children.forEach((child) => content.append(renderCard(child)));
    details.append(content);
    return details;
  }

  function renderControl(card) {
    const wrap = el("div", "card-control");
    if (card.type === "toggle") {
      const button = el("button", "toggle" + (state[card.key] ? " on" : ""));
      button.type = "button";
      button.setAttribute("aria-pressed", String(Boolean(state[card.key])));
      button.setAttribute("aria-label", card.title);
      button.addEventListener("click", () => {
        state[card.key] = !state[card.key];
        render();
      });
      wrap.append(button);
    }
    if (card.type === "select") {
      const selectNode = el("select", "select-control");
      card.options.forEach((option, index) => {
        const optionNode = el("option", "", option);
        optionNode.value = String(index);
        selectNode.append(optionNode);
      });
      selectNode.value = String(state[card.key] || 0);
      selectNode.addEventListener("change", () => {
        state[card.key] = Number(selectNode.value);
        render();
      });
      wrap.append(selectNode);
    }
    if (card.type === "input") {
      const pair = el("div", "control-pair");
      const inputNode = el("input", "input-control");
      inputNode.value = state[card.key];
      inputNode.inputMode = "numeric";
      inputNode.addEventListener("input", () => {
        state[card.key] = inputNode.value;
      });
      pair.append(inputNode, el("span", "unit", card.unit || ""));
      wrap.append(pair);
    }
    if (card.type === "slider") {
      const pair = el("div", "control-pair");
      const inputNode = el("input", "range-control");
      inputNode.type = "range";
      inputNode.min = card.min;
      inputNode.max = card.max;
      inputNode.value = state[card.key];
      const value = el("span", "range-value", String(state[card.key]));
      inputNode.addEventListener("input", () => {
        state[card.key] = Number(inputNode.value);
        value.textContent = inputNode.value;
      });
      pair.append(inputNode, value);
      wrap.append(pair);
    }
    return wrap;
  }

  function renderInfoBar(text, tone, spacing) {
    const node = el("div", "info-bar " + (tone || "info") + spacingClass(spacing));
    const symbol = tone === "warning" ? "!" : tone === "success" ? "OK" : "i";
    node.append(el("span", "symbol", symbol), el("span", "", text));
    return node;
  }

  function renderBadge(text, success) {
    return el("span", "badge" + (success ? " success" : ""), text);
  }

  function renderPerks(items) {
    const list = el("ul", "perk-list");
    items.forEach((item) => list.append(el("li", "", item)));
    return list;
  }

  function renderLink(label, iconName) {
    const link = el("a", "link-card");
    link.href = "#";
    const icon = el("img", "nav-icon");
    icon.src = iconPath(iconName);
    icon.alt = "";
    link.append(icon, el("span", "", label));
    return link;
  }

  function renderFooterNote() {
    const footer = el("div", "footer-note");
    footer.append(el("span", "star", "*"), el("span", "", "앱이 마음에 드시나요? GitHub에서 별을 주시거나,\nMicrosoft Store에 리뷰를 남겨주세요!"), renderLink("GitHub 저장소", "info.svg"));
    return footer;
  }

  function boolText(value) {
    return value ? "사용 중" : "사용 안함";
  }

  function spacingClass(spacing) {
    if (!spacing) return "";
    return " " + spacing;
  }

  window.addEventListener("hashchange", render);
  render();
})();
