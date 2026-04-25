// Copyright (C) 2024-2026 The FluentFlyout Authors
// SPDX-License-Identifier: GPL-3.0-or-later

using FluentFlyout.Classes;
using FluentFlyout.Classes.Settings;
using FluentFlyout.Classes.Utils;
using FluentFlyoutWPF.Classes;
using FluentFlyoutWPF.Classes.Display;
using FluentFlyoutWPF.Classes.Utils;
using MicaWPF.Controls;
using MicaWPF.Core.Extensions;
using System.Runtime.InteropServices;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Controls.Primitives;
using System.Windows.Input;
using System.Windows.Media;
using System.Windows.Media.Imaging;
using Windows.Media.Control;
using WindowsMediaController;
using static WindowsMediaController.MediaManager;

namespace FluentFlyoutDisplayHost;

public partial class MediaFlyoutWindow : MicaWindow
{
    private static readonly NLog.Logger Logger = NLog.LogManager.GetCurrentClassLogger();
    private readonly IDisplayHost _displayHost;
    private readonly System.Threading.Timer _positionTimer;
    private CancellationTokenSource _hideDelay = new();
    private bool _isActive;
    private bool _isDragging;
    private bool _isHiding = true;
    private bool _mediaSessionSupportsSeekbar;
    private int _themeOption = SettingsManager.Current.AppTheme;
    private bool _acrylicEnabled;

    public MediaFlyoutWindow(IDisplayHost displayHost)
    {
        _displayHost = displayHost;
        DataContext = SettingsManager.Current;
        WindowHelper.SetNoActivate(this);
        InitializeComponent();
        WindowHelper.SetTopmost(this);

        WindowStartupLocation = WindowStartupLocation.Manual;
        Left = -Width - 20;
        CustomWindowChrome.CaptionHeight = 0;
        CustomWindowChrome.UseAeroCaptionButtons = false;
        CustomWindowChrome.GlassFrameThickness = new Thickness(0);
        _positionTimer = new System.Threading.Timer(SeekbarUpdateUi, null, Timeout.Infinite, Timeout.Infinite);
        Closed += (_, _) =>
        {
            _hideDelay.Cancel();
            _hideDelay.Dispose();
            _positionTimer.Dispose();
        };
    }

    public async void ShowMediaFlyout(bool toggleMode = false, bool forceShow = false)
    {
        var session = _displayHost.GetTidalSession();
        if (session?.ControlSession == null ||
            (!forceShow && !SettingsManager.Current.MediaFlyoutEnabled) ||
            false)
        {
            return;
        }

        if (toggleMode && Visibility == Visibility.Visible && !_isHiding)
        {
            await HideWithAnimationAsync();
            return;
        }

        UpdateUI(session);
        if (SettingsManager.Current.SeekbarEnabled)
            HandlePlayBackState(session.ControlSession.GetPlaybackInfo().PlaybackStatus);

        if (_isHiding)
        {
            _isHiding = false;
            _displayHost.OpenAnimation(this);
        }

        _hideDelay.Cancel();
        _hideDelay.Dispose();
        _hideDelay = new CancellationTokenSource();
        var token = _hideDelay.Token;

        Visibility = Visibility.Visible;
        WindowHelper.SetTopmost(this);

        try
        {
            while (!token.IsCancellationRequested)
            {
                await Task.Delay(100, token);
                if (!IsMouseOver && !SettingsManager.Current.MediaFlyoutAlwaysDisplay)
                {
                    await Task.Delay(SettingsManager.Current.Duration, token);
                    if (!IsMouseOver)
                    {
                        await HideWithAnimationAsync();
                        break;
                    }
                }
            }
        }
        catch (TaskCanceledException)
        {
            // 새 표시 요청이 들어오면 기존 닫기 타이머만 취소한다.
        }
    }

    public void RefreshFromCurrentSession()
    {
        if (Visibility != Visibility.Visible)
            return;

        var session = _displayHost.GetTidalSession();
        if (session?.ControlSession == null)
            return;

        UpdateUI(session);
        if (SettingsManager.Current.SeekbarEnabled)
            HandlePlayBackState(session.ControlSession.GetPlaybackInfo().PlaybackStatus);
    }

    private async Task HideWithAnimationAsync()
    {
        _displayHost.CloseAnimation(this);
        _isHiding = true;
        _hideDelay.Cancel();
        await Task.Delay(_displayHost.getDuration());
        if (_isHiding)
        {
            Hide();
            if (SettingsManager.Current.SeekbarEnabled)
                HandlePlayBackState(GlobalSystemMediaTransportControlsSessionPlaybackStatus.Paused);
        }
    }

    private void UpdateUI(MediaSession mediaSession)
    {
        if (mediaSession.ControlSession == null)
            return;

        var controlSession = mediaSession.ControlSession;

        Dispatcher.Invoke(() =>
        {
            UpdateUILayout();
            UpdateMediaFlyoutCloseButtonVisibility();
            this.EnableBackdrop();

            var playback = controlSession.GetPlaybackInfo();
            ControlPlayPause.IsEnabled = true;
            ControlPlayPause.Opacity = 1;
            SymbolPlayPause.Symbol = playback.Controls.IsPauseEnabled
                ? Wpf.Ui.Controls.SymbolRegular.Pause16
                : Wpf.Ui.Controls.SymbolRegular.Play16;

            ControlBack.IsEnabled = playback.Controls.IsPreviousEnabled;
            ControlForward.IsEnabled = playback.Controls.IsNextEnabled;
            ControlBack.Opacity = playback.Controls.IsPreviousEnabled ? 1 : 0.35;
            ControlForward.Opacity = playback.Controls.IsNextEnabled ? 1 : 0.35;

            UpdateRepeatShuffle(playback);
            UpdatePlayerInfo(mediaSession);
            UpdateBackgroundVisibility();

            if (SettingsManager.Current.MediaFlyoutAcrylicWindowEnabled != _acrylicEnabled ||
                SettingsManager.Current.AppTheme != _themeOption)
            {
                _acrylicEnabled = SettingsManager.Current.MediaFlyoutAcrylicWindowEnabled;
                _themeOption = SettingsManager.Current.AppTheme;
                if (SettingsManager.Current.MediaFlyoutAcrylicWindowEnabled)
                    WindowBlurHelper.EnableBlur(this);
                else
                    WindowBlurHelper.DisableBlur(this);
            }

            var media = TryGetMediaProperties(controlSession);
            BitmapImage? image = null;
            if (media != null)
            {
                SongTitle.Text = media.Title;
                SongArtist.Text = media.Artist;
                image = BitmapHelper.GetThumbnail(media.Thumbnail);
                SongImage.ImageSource = image;
                SongInfoStackPanel.ToolTip =
                    $"{media.Title}{(string.IsNullOrEmpty(media.Artist) ? string.Empty : "\n\n" + media.Artist)}";
            }

            SongImagePlaceholder.Visibility = SongImage.ImageSource == null ? Visibility.Visible : Visibility.Collapsed;
            UpdateBlurredBackground(image);
            UpdateSeekbar(controlSession);
        });
    }

    private void UpdateRepeatShuffle(GlobalSystemMediaTransportControlsSessionPlaybackInfo playback)
    {
        if (SettingsManager.Current.RepeatEnabled && !SettingsManager.Current.CompactLayout)
        {
            ControlRepeat.Visibility = Visibility.Visible;
            ControlRepeat.IsEnabled = playback.Controls.IsRepeatEnabled;
            ControlRepeat.Opacity = playback.Controls.IsRepeatEnabled ? 1 : 0.35;
            SymbolRepeat.Symbol = playback.AutoRepeatMode switch
            {
                Windows.Media.MediaPlaybackAutoRepeatMode.List => Wpf.Ui.Controls.SymbolRegular.ArrowRepeatAll24,
                Windows.Media.MediaPlaybackAutoRepeatMode.Track => Wpf.Ui.Controls.SymbolRegular.ArrowRepeat124,
                _ => Wpf.Ui.Controls.SymbolRegular.ArrowRepeatAllOff24,
            };
            SymbolRepeat.Opacity = playback.AutoRepeatMode == Windows.Media.MediaPlaybackAutoRepeatMode.None ? 0.5 : 1;
        }
        else
        {
            ControlRepeat.Visibility = Visibility.Collapsed;
        }

        if (SettingsManager.Current.ShuffleEnabled && !SettingsManager.Current.CompactLayout)
        {
            ControlShuffle.Visibility = Visibility.Visible;
            ControlShuffle.IsEnabled = playback.Controls.IsShuffleEnabled;
            ControlShuffle.Opacity = playback.Controls.IsShuffleEnabled ? 1 : 0.35;
            SymbolShuffle.Symbol = playback.IsShuffleActive == true
                ? Wpf.Ui.Controls.SymbolRegular.ArrowShuffle24
                : Wpf.Ui.Controls.SymbolRegular.ArrowShuffleOff24;
            SymbolShuffle.Opacity = playback.IsShuffleActive == true ? 1 : 0.5;
        }
        else
        {
            ControlShuffle.Visibility = Visibility.Collapsed;
        }
    }

    private void UpdatePlayerInfo(MediaSession mediaSession)
    {
        if (SettingsManager.Current.PlayerInfoEnabled && !SettingsManager.Current.CompactLayout)
        {
            MediaIdStackPanel.Visibility = Visibility.Visible;
            var (title, icon) = MediaPlayerData.getMediaPlayerData(mediaSession.Id);
            MediaId.Text = title;
            MediaIdIcon.Source = icon;
            MediaIdIcon.Visibility = icon == null ? Visibility.Collapsed : Visibility.Visible;
        }
        else
        {
            MediaIdStackPanel.Visibility = Visibility.Collapsed;
        }
    }

    private void UpdateBackgroundVisibility()
    {
        BackgroundImageStyle1.Visibility = SettingsManager.Current.MediaFlyoutBackgroundBlur == 1 ? Visibility.Visible : Visibility.Collapsed;
        BackgroundImageStyle2.Visibility = SettingsManager.Current.MediaFlyoutBackgroundBlur == 2 ? Visibility.Visible : Visibility.Collapsed;
        BackgroundImageStyle3.Visibility = SettingsManager.Current.MediaFlyoutBackgroundBlur == 3 ? Visibility.Visible : Visibility.Collapsed;
    }

    private void UpdateBlurredBackground(BitmapImage? image)
    {
        if (SettingsManager.Current.MediaFlyoutBackgroundBlur == 0 || image == null)
            return;

        var croppedImage = CropToSquare(image);
        switch (SettingsManager.Current.MediaFlyoutBackgroundBlur)
        {
            case 1:
                BackgroundImageStyle1.Source = croppedImage;
                break;
            case 2:
                BackgroundImageStyle2.Source = croppedImage;
                break;
            case 3:
                BackgroundImageStyle3.Source = croppedImage;
                break;
        }
    }

    private void UpdateSeekbar(GlobalSystemMediaTransportControlsSession controlSession)
    {
        if (!SettingsManager.Current.SeekbarEnabled)
            return;

        var timeline = controlSession.GetTimelineProperties();
        var supportsSeekbar = timeline.MaxSeekTime.TotalSeconds >= 1.0;
        if (_mediaSessionSupportsSeekbar != supportsSeekbar)
        {
            _mediaSessionSupportsSeekbar = supportsSeekbar;
            UpdateUILayout();
        }

        if (supportsSeekbar)
        {
            Seekbar.Maximum = timeline.MaxSeekTime.TotalSeconds;
            SeekbarMaxDuration.Text = timeline.MaxSeekTime.ToString(timeline.MaxSeekTime.Hours > 0 ? @"hh\:mm\:ss" : @"mm\:ss");
        }
    }

    private void UpdateUILayout()
    {
        var extraWidth = SettingsManager.Current.RepeatEnabled ? 36 : 0;
        extraWidth += SettingsManager.Current.ShuffleEnabled ? 36 : 0;
        extraWidth += SettingsManager.Current.PlayerInfoEnabled ? 72 : 72;
        var extraHeight = SettingsManager.Current.SeekbarEnabled && _mediaSessionSupportsSeekbar ? 36 : 0;

        if (SettingsManager.Current.CompactLayout)
        {
            Height = 60 + extraHeight;
            Width = 400;
            BodyStackPanel.Orientation = Orientation.Horizontal;
            BodyStackPanel.Width = 300;
            ControlsStackPanel.Margin = new Thickness(0);
            ControlsStackPanel.Width = 104;
            MediaIdStackPanel.Visibility = Visibility.Collapsed;
            SongImageBorder.Margin = new Thickness(0);
            SongImageBorder.Height = 36;
            SongInfoStackPanel.Margin = new Thickness(8, 0, 0, 0);
            SongInfoStackPanel.Width = SettingsManager.Current.MediaFlyoutAlwaysDisplay ? 146 : 182;
            if (SettingsManager.Current.MediaFlyoutAlwaysDisplay)
                ControlsStackPanel.Width += 44;
        }
        else
        {
            Height = 112 + extraHeight;
            Width = 310 - 72 + extraWidth;
            BodyStackPanel.Orientation = Orientation.Vertical;
            BodyStackPanel.Width = 194 - 72 + extraWidth;
            ControlsStackPanel.Margin = new Thickness(12, 8, 0, 0);
            ControlsStackPanel.Width = 184 - 72 + extraWidth;
            SongImageBorder.Margin = new Thickness(6);
            SongImageBorder.Height = 78;
            SongInfoStackPanel.Margin = new Thickness(12, 0, 0, 0);
            SongInfoStackPanel.Width = 182 - 72 + extraWidth;
        }

        SongTitle.HorizontalAlignment = SettingsManager.Current.CenterTitleArtist ? HorizontalAlignment.Center : HorizontalAlignment.Left;
        SongArtist.HorizontalAlignment = SettingsManager.Current.CenterTitleArtist ? HorizontalAlignment.Center : HorizontalAlignment.Left;
        SeekbarWrapper.Visibility = SettingsManager.Current.SeekbarEnabled ? Visibility.Visible : Visibility.Collapsed;
    }

    private void UpdateMediaFlyoutCloseButtonVisibility()
    {
        MediaFlyoutCloseButton.Visibility = SettingsManager.Current.MediaFlyoutAlwaysDisplay && !SettingsManager.Current.CompactLayout ? Visibility.Visible : Visibility.Collapsed;
        ControlClose.Visibility = SettingsManager.Current.MediaFlyoutAlwaysDisplay && SettingsManager.Current.CompactLayout ? Visibility.Visible : Visibility.Collapsed;
    }

    private async void Back_Click(object sender, RoutedEventArgs e)
    {
        await (_displayHost.GetTidalSession()?.ControlSession?.TrySkipPreviousAsync()?.AsTask() ?? Task.CompletedTask);
        await RefreshAfterCommandAsync();
    }

    private async void PlayPause_Click(object sender, RoutedEventArgs e)
    {
        await (_displayHost.GetTidalSession()?.ControlSession?.TryTogglePlayPauseAsync()?.AsTask() ?? Task.CompletedTask);
        await RefreshAfterCommandAsync();
    }

    private async void Forward_Click(object sender, RoutedEventArgs e)
    {
        await (_displayHost.GetTidalSession()?.ControlSession?.TrySkipNextAsync()?.AsTask() ?? Task.CompletedTask);
        await RefreshAfterCommandAsync();
    }

    private async void Repeat_Click(object sender, RoutedEventArgs e)
    {
        var session = _displayHost.GetTidalSession()?.ControlSession;
        if (session == null)
            return;

        var mode = session.GetPlaybackInfo().AutoRepeatMode switch
        {
            Windows.Media.MediaPlaybackAutoRepeatMode.None => Windows.Media.MediaPlaybackAutoRepeatMode.List,
            Windows.Media.MediaPlaybackAutoRepeatMode.List => Windows.Media.MediaPlaybackAutoRepeatMode.Track,
            _ => Windows.Media.MediaPlaybackAutoRepeatMode.None,
        };
        await session.TryChangeAutoRepeatModeAsync(mode);
        UpdateUI(_displayHost.GetTidalSession()!);
    }

    private async void Shuffle_Click(object sender, RoutedEventArgs e)
    {
        var session = _displayHost.GetTidalSession()?.ControlSession;
        if (session == null)
            return;

        var active = session.GetPlaybackInfo().IsShuffleActive == true;
        await session.TryChangeShuffleActiveAsync(!active);
        UpdateUI(_displayHost.GetTidalSession()!);
    }

    private async Task RefreshAfterCommandAsync()
    {
        RefreshFromCurrentSession();
        await Task.Delay(120);
        RefreshFromCurrentSession();
    }

    private async void MediaFlyoutCloseButton_Click(object sender, RoutedEventArgs e)
    {
        await HideWithAnimationAsync();
    }

    private void Seekbar_OnPreviewMouseLeftButtonDown(object sender, MouseButtonEventArgs e)
    {
        if (_isDragging) return;
        _isDragging = true;

        var slider = (Slider)sender;
        var clickPosition = e.GetPosition(slider);
        var thumbWidth = slider.Template.FindName("Thumb", slider) is Thumb thumb ? thumb.ActualWidth : 0;
        var ratio = (clickPosition.X - thumbWidth / 2) / (slider.ActualWidth - thumbWidth);
        ratio = Math.Max(0, Math.Min(1, ratio));
        var targetSeconds = ratio * slider.Maximum;
        if (targetSeconds == 0) targetSeconds = 1;
        Seekbar.Value = targetSeconds;
    }

    private async void Seekbar_OnPreviewMouseLeftButtonUp(object sender, MouseButtonEventArgs e)
    {
        var session = _displayHost.GetTidalSession()?.ControlSession;
        if (session != null)
        {
            var seekPosition = TimeSpan.FromSeconds(Seekbar.Value);
            if (seekPosition == TimeSpan.Zero) seekPosition = TimeSpan.FromSeconds(1);
            await session.TryChangePlaybackPositionAsync(seekPosition.Ticks);
        }
        _isDragging = false;
    }

    private void Seekbar_OnValueChanged(object sender, RoutedPropertyChangedEventArgs<double> e)
    {
        if (!_isDragging) return;
        var timespan = TimeSpan.FromSeconds(e.NewValue);
        SeekbarCurrentDuration.Text = timespan.ToString(timespan.Hours > 0 ? @"hh\:mm\:ss" : @"mm\:ss");
    }

    private void SeekbarUpdateUi(object? sender)
    {
        if (!SettingsManager.Current.SeekbarEnabled || Visibility != Visibility.Visible || _isDragging)
            return;

        var session = _displayHost.GetTidalSession()?.ControlSession;
        if (session == null)
            return;

        var timeline = session.GetTimelineProperties();
        var pos = timeline.Position + (DateTime.Now - timeline.LastUpdatedTime.DateTime);
        Dispatcher.Invoke(() =>
        {
            Seekbar.Value = pos.TotalSeconds;
            SeekbarCurrentDuration.Text = pos.ToString(pos.Hours > 0 ? @"hh\:mm\:ss" : @"mm\:ss");
        });
    }

    private void HandlePlayBackState(GlobalSystemMediaTransportControlsSessionPlaybackStatus? status)
    {
        if (status == GlobalSystemMediaTransportControlsSessionPlaybackStatus.Playing)
        {
            if (_isActive) return;
            _isActive = true;
            _positionTimer.Change(0, 300);
        }
        else
        {
            if (!_isActive) return;
            _isActive = false;
            _positionTimer.Change(Timeout.Infinite, Timeout.Infinite);
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

    private static CroppedBitmap? CropToSquare(BitmapImage? sourceImage)
    {
        if (sourceImage == null)
            return null;

        var size = (int)Math.Min(sourceImage.PixelWidth, sourceImage.PixelHeight);
        var x = (sourceImage.PixelWidth - size) / 2;
        var y = (sourceImage.PixelHeight - size) / 2;
        var croppedBitmap = new CroppedBitmap(sourceImage, new Int32Rect(x, y, size, size));
        croppedBitmap.Freeze();
        return croppedBitmap;
    }
}
