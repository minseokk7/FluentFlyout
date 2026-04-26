// Copyright (C) 2024-2026 The FluentFlyout Authors
// SPDX-License-Identifier: GPL-3.0-or-later

using FluentFlyout.Classes.Settings;
using System.Diagnostics;
using System.Windows;

namespace FluentFlyoutDisplayHost;

public partial class App : Application
{
    private static readonly NLog.Logger Logger = NLog.LogManager.GetCurrentClassLogger();

    protected override void OnStartup(StartupEventArgs e)
    {
        AppDomain.CurrentDomain.UnhandledException += (_, args) =>
        {
            Logger.Error(args.ExceptionObject as Exception, "DisplayHost에서 처리되지 않은 예외가 발생했습니다.");
            NLog.LogManager.Flush();
        };

        DispatcherUnhandledException += (_, args) =>
        {
            Logger.Error(args.Exception, "DisplayHost UI 스레드에서 처리되지 않은 예외가 발생했습니다.");
            NLog.LogManager.Flush();
        };

        base.OnStartup(e);
        Logger.Info("DisplayHost startup begin");
        StartParentProcessMonitor(e.Args);

        try
        {
            new SettingsManager().RestoreSettings();
            SettingsManager.Current.IsPremiumUnlocked = true;
            SettingsManager.Current.IsStoreVersion = false;
        }
        catch (Exception ex)
        {
            Logger.Error(ex, "원본 설정을 불러오지 못했습니다.");
        }

        Logger.Info("DisplayHost creating main window");
        MainWindow = new DisplayHostWindow();
        Logger.Info("DisplayHost showing main window");
        MainWindow.Show();
        Logger.Info("DisplayHost main window show returned");
    }

    private void StartParentProcessMonitor(string[] args)
    {
        var parentPid = ParseParentPid(args);
        if (parentPid is null)
        {
            return;
        }

        Task.Run(async () =>
        {
            while (true)
            {
                await Task.Delay(1000).ConfigureAwait(false);
                if (IsProcessAlive(parentPid.Value))
                {
                    continue;
                }

                Logger.Info("부모 FluentFlyout 프로세스가 종료되어 DisplayHost를 종료합니다.");
                await Dispatcher.InvokeAsync(Shutdown);
                break;
            }
        });
    }

    private static int? ParseParentPid(string[] args)
    {
        for (var i = 0; i < args.Length - 1; i++)
        {
            if (args[i].Equals("--parent-pid", StringComparison.OrdinalIgnoreCase)
                && int.TryParse(args[i + 1], out var pid)
                && pid > 0)
            {
                return pid;
            }
        }

        return null;
    }

    private static bool IsProcessAlive(int pid)
    {
        try
        {
            using var process = Process.GetProcessById(pid);
            return !process.HasExited;
        }
        catch (ArgumentException)
        {
            return false;
        }
        catch (InvalidOperationException)
        {
            return false;
        }
    }
}
