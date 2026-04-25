// Copyright © 2024-2026 The FluentFlyout Authors
// SPDX-License-Identifier: GPL-3.0-or-later

using FluentFlyoutWPF.Classes.Utils;
using MicaWPF.Controls;

namespace FluentFlyoutWPF.Classes.Display;

public interface IDisplayHost
{
    WindowsMediaController.MediaManager MediaManager { get; }

    WindowsMediaController.MediaManager.MediaSession? GetTidalSession();

    int getDuration();

    void OpenAnimation(MicaWindow window, bool alwaysBottom = false, MonitorUtil.MonitorInfo? selectedMonitor = null);

    void CloseAnimation(MicaWindow window, bool alwaysBottom = false, MonitorUtil.MonitorInfo? selectedMonitor = null);

    void ShowMediaFlyout(bool toggleMode = false, bool forceShow = false);

    void RecreateTaskbarWindow();
}
