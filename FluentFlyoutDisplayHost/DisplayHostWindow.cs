// Copyright © 2024-2026 The FluentFlyout Authors
// SPDX-License-Identifier: GPL-3.0-or-later

using FluentFlyout.Classes;
using FluentFlyout.Classes.Settings;
using FluentFlyout.Classes.Utils;
using FluentFlyout.Windows;
using FluentFlyoutWPF.Classes;
using FluentFlyoutWPF.Classes.Display;
using FluentFlyoutWPF.Classes.Utils;
using FluentFlyoutWPF.Windows;
using MicaWPF.Controls;
using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Windows;
using System.Windows.Input;
using System.Windows.Media.Animation;
using System.Windows.Media.Imaging;
using Windows.Media.Control;
using WindowsMediaController;
using static WindowsMediaController.MediaManager;

namespace FluentFlyoutDisplayHost;

public sealed class DisplayHostWindow : Window, IDisplayHost
{
    private static readonly NLog.Logger Logger = NLog.LogManager.GetCurrentClassLogger();
    private const int WH_KEYBOARD_LL = 13;
    private const int WM_KEYUP = 0x0101;
    private const string ShowMediaFlyoutEventName = @"Local\FluentFlyout_DisplayHost_ShowMediaFlyout";
    private const string ShowNextUpFlyoutEventName = @"Local\FluentFlyout_DisplayHost_ShowNextUpFlyout";
    private const string ShowLockKeysFlyoutEventName = @"Local\FluentFlyout_DisplayHost_ShowLockKeysFlyout";
    private const string ReloadSettingsEventName = @"Local\FluentFlyout_DisplayHost_ReloadSettings";

    private delegate IntPtr LowLevelKeyboardProc(int nCode, IntPtr wParam, IntPtr lParam);

    private readonly LowLevelKeyboardProc _hookProc;
    private readonly System.Threading.Timer _positionTimer;
    private IntPtr _hookId;
    private LockWindow? _lockWindow;
    private MediaFlyoutWindow? _mediaFlyoutWindow;
    private NextUpWindow? _nextUpWindow;
    private TaskbarWindow? _taskbarWindow;
    private string _currentTitle = string.Empty;
    private string _lastActiveSessionId = string.Empty;
    private string _lastTaskbarTitle = string.Empty;
    private string _lastTaskbarArtist = string.Empty;
    private BitmapImage? _lastTaskbarThumbnail;
    private int _lastTaskbarThumbnailHash;
    private CancellationTokenSource? _nextUpDelay;

    public WindowsMediaController.MediaManager MediaManager { get; } = new();

    public DisplayHostWindow()
    {
        Logger.Info("DisplayHostWindow constructor begin");
        Width = 1;
        Height = 1;
        ShowInTaskbar = false;
        WindowStyle = WindowStyle.None;
        ResizeMode = ResizeMode.NoResize;
        Opacity = 0;
        Left = -10000;
        Top = -10000;

        DataContext = SettingsManager.Current;
        _hookProc = HookCallback;
        _positionTimer = new System.Threading.Timer(_ => { }, null, Timeout.Infinite, Timeout.Infinite);

        Loaded += OnLoaded;
        Closed += (_, _) => Cleanup();
        Logger.Info("DisplayHostWindow constructor end");
    }

    public MediaSession? GetTidalSession()
    {
        if (SettingsManager.Current.ExclusiveTidalMode)
        {
            return MediaManager.CurrentMediaSessions.Values.FirstOrDefault(session =>
                session.Id.Contains("TIDAL", StringComparison.OrdinalIgnoreCase));
        }

        if (!string.IsNullOrEmpty(_lastActiveSessionId)
            && MediaManager.CurrentMediaSessions.TryGetValue(_lastActiveSessionId, out var active))
        {
            return active;
        }

        var playing = MediaManager.CurrentMediaSessions.Values.FirstOrDefault(session =>
            session.ControlSession?.GetPlaybackInfo()?.PlaybackStatus == GlobalSystemMediaTransportControlsSessionPlaybackStatus.Playing);

        return playing ?? MediaManager.CurrentMediaSessions.Values.FirstOrDefault();
    }

    public int getDuration()
    {
        return SettingsManager.Current.FlyoutAnimationSpeed switch
        {
            0 => 0,
            1 => 150,
            2 => 300,
            3 => 450,
            4 => 600,
            _ => 900,
        };
    }

    public void OpenAnimation(MicaWindow window, bool alwaysBottom = false, MonitorUtil.MonitorInfo? selectedMonitor = null)
    {
        var storyboard = ((BeginStoryboard)((EventTrigger)window.Triggers[0]).Actions[0]).Storyboard;
        var moveAnimation = (DoubleAnimation)storyboard.Children[0];
        var opacityAnimation = (DoubleAnimation)storyboard.Children[1];
        var monitor = selectedMonitor ?? MonitorUtil.GetSelectedMonitor(SettingsManager.Current.FlyoutSelectedMonitor);
        var workArea = monitor.workArea;

        WindowHelper.SetVisibility(window, false);
        WindowHelper.SetPosition(window, workArea.Left, workArea.Top);
        var windowWidth = window.ActualWidth > 0 ? window.ActualWidth : window.Width;
        var windowHeight = window.ActualHeight > 0 ? window.ActualHeight : window.Height;

        double left;
        if (alwaysBottom)
        {
            left = workArea.Left + workArea.Width / 2 - windowWidth / 2;
            moveAnimation.To = workArea.Top + workArea.Height - windowHeight - 16;
            moveAnimation.From = SettingsManager.Current.FlyoutAnimationSpeed == 0
                ? moveAnimation.To
                : workArea.Top + workArea.Height - windowHeight + 4;
        }
        else
        {
            (left, moveAnimation.From, moveAnimation.To) = PlacementFor(windowWidth, windowHeight, workArea);
        }

        WindowHelper.SetPosition(window, left, moveAnimation.From!.Value);
        moveAnimation.From *= 96.0 / monitor.dpiY;
        moveAnimation.To *= 96.0 / monitor.dpiY;

        var duration = new Duration(TimeSpan.FromMilliseconds(getDuration()));
        moveAnimation.Duration = duration;
        opacityAnimation.From = SettingsManager.Current.FlyoutAnimationSpeed == 0 ? 1 : 0;
        opacityAnimation.To = 1;
        opacityAnimation.Duration = duration;
        storyboard.Begin(window);
        WindowHelper.SetVisibility(window, true);
        WindowHelper.SetTopmost(window);
    }

    public void CloseAnimation(MicaWindow window, bool alwaysBottom = false, MonitorUtil.MonitorInfo? selectedMonitor = null)
    {
        var storyboard = ((BeginStoryboard)((EventTrigger)window.Triggers[0]).Actions[0]).Storyboard;
        var moveAnimation = (DoubleAnimation)storyboard.Children[0];
        var opacityAnimation = (DoubleAnimation)storyboard.Children[1];
        var monitor = selectedMonitor ?? MonitorUtil.GetSelectedMonitor(SettingsManager.Current.FlyoutSelectedMonitor);
        var workArea = monitor.workArea;
        var windowWidth = window.ActualWidth > 0 ? window.ActualWidth : window.Width;
        var windowHeight = window.ActualHeight > 0 ? window.ActualHeight : window.Height;

        if (alwaysBottom)
        {
            moveAnimation.From = workArea.Top + workArea.Height - windowHeight - 16;
            moveAnimation.To = SettingsManager.Current.FlyoutAnimationSpeed == 0
                ? moveAnimation.From
                : workArea.Top + workArea.Height - windowHeight + 4;
        }
        else
        {
            (_, moveAnimation.From, moveAnimation.To) = PlacementFor(windowWidth, windowHeight, workArea);
        }

        moveAnimation.From *= 96.0 / monitor.dpiY;
        moveAnimation.To *= 96.0 / monitor.dpiY;

        var duration = new Duration(TimeSpan.FromMilliseconds(getDuration()));
        moveAnimation.Duration = duration;
        opacityAnimation.From = 1;
        opacityAnimation.To = SettingsManager.Current.FlyoutAnimationSpeed == 0 ? 1 : 0;
        opacityAnimation.Duration = duration;
        storyboard.Begin(window);
    }

    public void ShowMediaFlyout(bool toggleMode = false, bool forceShow = false)
    {
        Dispatcher.Invoke(() =>
        {
            try
            {
                if (GetTidalSession() == null ||
                    (!forceShow && !SettingsManager.Current.MediaFlyoutEnabled))
                {
                    Logger.Info("MediaFlyout show skipped. forceShow={0}", forceShow);
                    return;
                }

                _mediaFlyoutWindow ??= new MediaFlyoutWindow(this);
                Logger.Info("MediaFlyout show requested. toggleMode={0}, forceShow={1}", toggleMode, forceShow);
                _mediaFlyoutWindow.ShowMediaFlyout(toggleMode, forceShow);

                if (_nextUpWindow != null)
                {
                    _nextUpWindow.Close();
                    _nextUpWindow = null;
                }
            }
            catch (Exception ex)
            {
                Logger.Error(ex, "미디어 Flyout을 표시하지 못했습니다.");
            }
        });
    }

    public void RecreateTaskbarWindow()
    {
        Dispatcher.Invoke(() =>
        {
            try
            {
                _taskbarWindow?.Close();
                _taskbarWindow = new TaskbarWindow();
                UpdateTaskbar();
            }
            catch (Exception ex)
            {
                Logger.Error(ex, "작업표시줄 위젯 창을 다시 만들지 못했습니다.");
            }
        });
    }

    private void OnLoaded(object sender, RoutedEventArgs e)
    {
        Logger.Info("DisplayHostWindow loaded begin");
        // 호스트 창을 Hide()하면 WPF가 표시 창이 없다고 판단해 프로세스가 종료될 수 있다.
        // 원본 표시 창들은 별도로 띄우고, 이 창은 1x1 투명 오프스크린 메시지 펌프로 유지한다.
        Left = -10000;
        Top = -10000;
        Width = 1;
        Height = 1;
        Opacity = 0;
        Logger.Info("DisplayHost media manager start");
        MediaManager.Start();
        MediaManager.OnAnyMediaPropertyChanged += OnAnyMediaPropertyChanged;
        MediaManager.OnAnyPlaybackStateChanged += OnAnyPlaybackStateChanged;
        MediaManager.OnAnySessionClosed += OnAnySessionClosed;
        Logger.Info("DisplayHost keyboard hook install");
        _hookId = SetHook(_hookProc);
        StartCommandEventLoop();

        Logger.Info("DisplayHost creating TaskbarWindow");
        _taskbarWindow = new TaskbarWindow();
        Logger.Info("DisplayHost updating TaskbarWindow");
        UpdateTaskbar();
        Logger.Info("DisplayHostWindow loaded end");
    }

    private void StartCommandEventLoop()
    {
        Task.Run(() =>
        {
            try
            {
                using var showMediaEvent = new EventWaitHandle(false, EventResetMode.AutoReset, ShowMediaFlyoutEventName);
                using var showNextUpEvent = new EventWaitHandle(false, EventResetMode.AutoReset, ShowNextUpFlyoutEventName);
                using var showLockKeysEvent = new EventWaitHandle(false, EventResetMode.AutoReset, ShowLockKeysFlyoutEventName);
                using var reloadSettingsEvent = new EventWaitHandle(false, EventResetMode.AutoReset, ReloadSettingsEventName);
                var handles = new WaitHandle[] { showMediaEvent, showNextUpEvent, showLockKeysEvent, reloadSettingsEvent };
                while (true)
                {
                    var signal = WaitHandle.WaitAny(handles);
                    Dispatcher.Invoke(() =>
                    {
                        switch (signal)
                        {
                            case 0:
                                Logger.Info("DisplayHost command received: show media flyout");
                                ShowMediaFlyout(toggleMode: true, forceShow: true);
                                break;
                            case 1:
                                Logger.Info("DisplayHost command received: show next-up flyout");
                                ShowNextUpFlyoutForCurrentSession();
                                break;
                            case 2:
                                Logger.Info("DisplayHost command received: show lock-keys flyout");
                                ShowLockKeysTestFlyout();
                                break;
                            case 3:
                                Logger.Info("DisplayHost command received: reload settings");
                                ReloadSettingsWithoutRestart();
                                break;
                        }
                    });
                }
            }
            catch (Exception ex)
            {
                Logger.Error(ex, "DisplayHost 명령 이벤트 루프가 중단되었습니다.");
            }
        });
    }

    private void ReloadSettingsWithoutRestart()
    {
        try
        {
            var oldControlsPosition = SettingsManager.Current.TaskbarWidgetControlsPosition;

            new SettingsManager().RestoreSettings();
            SettingsManager.Current.IsPremiumUnlocked = true;
            SettingsManager.Current.IsStoreVersion = false;

            DataContext = SettingsManager.Current;
            _taskbarWindow?.ApplySettingsReload(oldControlsPosition != SettingsManager.Current.TaskbarWidgetControlsPosition);
            UpdateTaskbar();
            _mediaFlyoutWindow?.RefreshFromCurrentSession();
        }
        catch (Exception ex)
        {
            Logger.Error(ex, "설정을 다시 읽는 중 오류가 발생했습니다.");
        }
    }

    private void ShowNextUpFlyoutForCurrentSession()
    {
        UpdateTaskbar();
        if (_nextUpWindow != null)
        {
            WindowHelper.SetVisibility(_nextUpWindow, false);
            _nextUpWindow.Close();
            _nextUpWindow = null;
        }

        if (string.IsNullOrWhiteSpace(_lastTaskbarTitle) || _lastTaskbarThumbnail == null)
        {
            Logger.Info("NextUp test skipped because no media thumbnail is available.");
            return;
        }

        _nextUpWindow = new NextUpWindow(_lastTaskbarTitle, _lastTaskbarArtist, _lastTaskbarThumbnail);
        _nextUpWindow.Closed += (_, _) => _nextUpWindow = null;
    }

    private void ShowLockKeysTestFlyout()
    {
        _lockWindow ??= new LockWindow();
        _lockWindow.ShowLockFlyout(FindResource("LockWindow_CapsLock").ToString() ?? "Caps Lock", (GetKeyState(0x14) & 1) != 0);
    }

    private void UpdateTaskbar()
    {
        var session = GetTidalSession();
        if (!MediaManager.IsStarted || session?.ControlSession == null)
        {
            _taskbarWindow?.UpdateUi("-", "-", null, GlobalSystemMediaTransportControlsSessionPlaybackStatus.Closed);
            return;
        }

        var media = TryGetMediaProperties(session.ControlSession);
        if (media == null)
            return;

        var playback = session.ControlSession.GetPlaybackInfo();
        var titleChanged = !string.IsNullOrWhiteSpace(_lastTaskbarTitle)
            && !string.Equals(_lastTaskbarTitle, media.Title, StringComparison.Ordinal);
        var thumbnailHash = media.Thumbnail == null ? 0 : BitmapHelper.GetStableThumbnailHash(media.Thumbnail);
        var thumbnailLooksStale = titleChanged
            && thumbnailHash != 0
            && thumbnailHash == _lastTaskbarThumbnailHash;

        var thumbnail = thumbnailLooksStale ? null : BitmapHelper.GetThumbnail(media.Thumbnail);
        if (thumbnail != null)
        {
            BitmapHelper.GetDominantColors(1);
            _lastTaskbarThumbnailHash = thumbnailHash;
        }

        _lastTaskbarTitle = media.Title;
        _lastTaskbarArtist = media.Artist;
        _lastTaskbarThumbnail = thumbnail;
        _taskbarWindow?.UpdateUi(media.Title, media.Artist, thumbnail, playback.PlaybackStatus, playback.Controls);
    }

    private void OnAnyPlaybackStateChanged(MediaSession mediaSession, GlobalSystemMediaTransportControlsSessionPlaybackInfo? playbackInfo = null)
    {
        _lastActiveSessionId = mediaSession.Id;
        PauseOtherMediaSessionsIfNeeded(mediaSession);
        UpdateTaskbar();
        Dispatcher.Invoke(() => _mediaFlyoutWindow?.RefreshFromCurrentSession());
    }

    private void OnAnySessionClosed(MediaSession mediaSession)
    {
        if (_lastActiveSessionId == mediaSession.Id)
            _lastActiveSessionId = string.Empty;
        UpdateTaskbar();
        Dispatcher.Invoke(() => _mediaFlyoutWindow?.RefreshFromCurrentSession());
    }

    private void OnAnyMediaPropertyChanged(MediaSession mediaSession, GlobalSystemMediaTransportControlsSessionMediaProperties mediaProperties)
    {
        _lastActiveSessionId = mediaSession.Id;
        PauseOtherMediaSessionsIfNeeded(mediaSession);
        var previousThumbnailHash = _lastTaskbarThumbnailHash;
        UpdateTaskbar();
        Dispatcher.Invoke(() => _mediaFlyoutWindow?.RefreshFromCurrentSession());

        if (!SettingsManager.Current.NextUpEnabled)
            return;

        if (mediaSession.ControlSession == null)
            return;

        if (SettingsManager.Current.ExclusiveTidalMode
            && !mediaSession.Id.Contains("TIDAL", StringComparison.OrdinalIgnoreCase))
            return;

        _nextUpDelay?.Cancel();
        var delay = new CancellationTokenSource();
        _nextUpDelay = delay;

        Dispatcher.Invoke(() =>
        {
            if (_nextUpWindow != null)
            {
                WindowHelper.SetVisibility(_nextUpWindow, false);
                _nextUpWindow.Close();
                _nextUpWindow = null;
            }
        });

        _ = Task.Run(async () =>
        {
            try
            {
                for (var attempt = 0; attempt < 8; attempt++)
                {
                    await Task.Delay(120, delay.Token);
                    if (delay.Token.IsCancellationRequested)
                        return;

                    UpdateTaskbar();

                    if (_lastTaskbarThumbnail != null
                        && (_lastTaskbarThumbnailHash != previousThumbnailHash || attempt == 7))
                    {
                        break;
                    }
                }

                var title = _lastTaskbarTitle;
                if (string.IsNullOrWhiteSpace(title) || _currentTitle == title)
                    return;

                var thumbnail = _lastTaskbarThumbnail;
                if (thumbnail == null)
                    return;

                var artist = _lastTaskbarArtist;
                Dispatcher.Invoke(() =>
                {
                    _nextUpWindow = new NextUpWindow(title, artist, thumbnail);
                    _nextUpWindow.Closed += (_, _) => _nextUpWindow = null;
                    _currentTitle = title;
                });
            }
            catch (TaskCanceledException)
            {
                // 빠르게 곡을 넘길 때는 마지막 이벤트만 사용한다.
            }
        });
    }

    private void PauseOtherMediaSessionsIfNeeded(MediaSession mediaSession)
    {
        if (!SettingsManager.Current.PauseOtherSessionsEnabled || mediaSession.ControlSession == null)
            return;

        var playback = mediaSession.ControlSession.GetPlaybackInfo();
        if (playback.PlaybackStatus != GlobalSystemMediaTransportControlsSessionPlaybackStatus.Playing)
            return;

        _ = Task.WhenAll(MediaManager.CurrentMediaSessions.Values.Select(session =>
        {
            try
            {
                if (session.Id == mediaSession.Id || session.ControlSession == null)
                    return Task.CompletedTask;

                var otherPlayback = session.ControlSession.GetPlaybackInfo();
                if (otherPlayback.PlaybackStatus == GlobalSystemMediaTransportControlsSessionPlaybackStatus.Playing)
                    return session.ControlSession.TryPauseAsync().AsTask();
            }
            catch (Exception ex)
            {
                Logger.Error(ex, "다른 미디어 세션을 일시 정지하지 못했습니다.");
            }

            return Task.CompletedTask;
        }));
    }

    private IntPtr HookCallback(int nCode, IntPtr wParam, IntPtr lParam)
    {
        if (nCode >= 0 && wParam == (IntPtr)WM_KEYUP)
        {
            var vkCode = Marshal.ReadInt32(lParam);
            var mediaKeysPressed = vkCode is 0xB0 or 0xB1 or 0xB2 or 0xB3;
            var volumeKeysPressed = vkCode is 0xAD or 0xAE or 0xAF;

            if (mediaKeysPressed || (!SettingsManager.Current.MediaFlyoutVolumeKeysExcluded && volumeKeysPressed))
            {
                ShowMediaFlyout();
            }

            var lockEvent = vkCode switch
            {
                0x14 => ("Caps Lock", (GetKeyState(0x14) & 1) != 0),
                0x90 => ("Num Lock", (GetKeyState(0x90) & 1) != 0),
                0x91 => ("Scroll Lock", (GetKeyState(0x91) & 1) != 0),
                0x2D when SettingsManager.Current.LockKeysInsertEnabled => ("Insert", true),
                _ => default,
            };

            if (lockEvent != default && SettingsManager.Current.LockKeysEnabled)
            {
                Dispatcher.Invoke(() =>
                {
                    _lockWindow ??= new LockWindow();
                    _lockWindow.ShowLockFlyout(lockEvent.Item1, lockEvent.Item2);
                });
            }
        }

        return CallNextHookEx(IntPtr.Zero, nCode, wParam, lParam);
    }

    private void Cleanup()
    {
        try
        {
            MediaManager.OnAnyMediaPropertyChanged -= OnAnyMediaPropertyChanged;
            MediaManager.OnAnyPlaybackStateChanged -= OnAnyPlaybackStateChanged;
            MediaManager.OnAnySessionClosed -= OnAnySessionClosed;
            _nextUpDelay?.Cancel();
            _nextUpDelay?.Dispose();
            _positionTimer.Dispose();
            if (_hookId != IntPtr.Zero)
                UnhookWindowsHookEx(_hookId);
            _lockWindow?.Close();
            _mediaFlyoutWindow?.Close();
            _nextUpWindow?.Close();
            _taskbarWindow?.Close();
            NLog.LogManager.Shutdown();
        }
        catch (Exception ex)
        {
            Logger.Error(ex, "표시 호스트 정리 중 오류가 발생했습니다.");
        }
    }

    private static GlobalSystemMediaTransportControlsSessionMediaProperties? TryGetMediaProperties(GlobalSystemMediaTransportControlsSession session)
    {
        try
        {
            return session.TryGetMediaPropertiesAsync().GetAwaiter().GetResult();
        }
        catch (COMException ex)
        {
            Logger.Error(ex, "미디어 정보를 가져오지 못했습니다.");
            return null;
        }
    }

    private static (double left, double from, double to) PlacementFor(double width, double height, Rect workArea)
    {
        var speedOff = SettingsManager.Current.FlyoutAnimationSpeed == 0;

        return SettingsManager.Current.Position switch
        {
            0 => (workArea.Left + 16, speedOff ? workArea.Top + workArea.Height - height - 16 : workArea.Top + workArea.Height - height + 4, workArea.Top + workArea.Height - height - 16),
            1 => (workArea.Left + workArea.Width / 2 - width / 2, speedOff ? workArea.Top + workArea.Height - height - 80 : workArea.Top + workArea.Height - height - 60, workArea.Top + workArea.Height - height - 80),
            2 => (workArea.Left + workArea.Width - width - 16, speedOff ? workArea.Top + workArea.Height - height - 16 : workArea.Top + workArea.Height - height + 4, workArea.Top + workArea.Height - height - 16),
            3 => (workArea.Left + 16, speedOff ? workArea.Top + 16 : workArea.Top - 4, workArea.Top + 16),
            4 => (workArea.Left + workArea.Width / 2 - width / 2, speedOff ? workArea.Top + 16 : workArea.Top - 4, workArea.Top + 16),
            5 => (workArea.Left + workArea.Width - width - 16, speedOff ? workArea.Top + 16 : workArea.Top - 4, workArea.Top + 16),
            _ => (workArea.Left + workArea.Width / 2 - width / 2, speedOff ? workArea.Top + workArea.Height - height - 80 : workArea.Top + workArea.Height - height - 60, workArea.Top + workArea.Height - height - 80),
        };
    }

    private static IntPtr SetHook(LowLevelKeyboardProc proc)
    {
        using var currentProcess = Process.GetCurrentProcess();
        using var currentModule = currentProcess.MainModule;
        return SetWindowsHookEx(WH_KEYBOARD_LL, proc, GetModuleHandle(currentModule.ModuleName), 0);
    }

    [DllImport("user32.dll", SetLastError = true)]
    private static extern IntPtr SetWindowsHookEx(int idHook, LowLevelKeyboardProc lpfn, IntPtr hMod, uint dwThreadId);

    [DllImport("user32.dll", SetLastError = true)]
    private static extern bool UnhookWindowsHookEx(IntPtr hhk);

    [DllImport("user32.dll", SetLastError = true)]
    private static extern IntPtr CallNextHookEx(IntPtr hhk, int nCode, IntPtr wParam, IntPtr lParam);

    [DllImport("user32.dll")]
    private static extern short GetKeyState(int nVirtKey);

    [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    private static extern IntPtr GetModuleHandle(string? lpModuleName);
}
