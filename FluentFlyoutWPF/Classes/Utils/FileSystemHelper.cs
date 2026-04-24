using FluentFlyout.Classes.Settings;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using Windows.Storage;
using System.Windows;
using System.IO;

namespace FluentFlyoutWPF.Classes.Utils
{
    internal class FileSystemHelper
    {
        public static string GetLogsPath()
        {
            string path;
            if (SettingsManager.Current.IsStoreVersion)
            {
                path = Path.Combine(ApplicationData.Current.LocalCacheFolder.Path, "Roaming", "FluentFlyout");
            }
            else
            {
                path = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), "FluentFlyout");
            }

            // Check if Logs subfolder exists, if not use the base path
            string logsSubPath = Path.Combine(path, "Logs");
            if (Directory.Exists(logsSubPath))
            {
                return logsSubPath;
            }

            // Ensure the directory exists
            if (!Directory.Exists(path))
            {
                try
                {
                    Directory.CreateDirectory(path);
                }
                catch { }
            }

            return path;
        }
    }
}
