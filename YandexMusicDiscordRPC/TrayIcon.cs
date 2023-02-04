using YandexMusicDiscordRPC.Properties;

namespace YandexMusicDiscordRPC
{
    public class TrayIcon
    {
        public static void Run() 
        {
            ContextMenuStrip trayMenu = new()
            {
                Items = {
                    new ToolStripMenuItem("Exit", Utils.IconExtractor.Extract("shell32.dll", 131, true)!.ToBitmap(), Exit_Click)
                }
            };

            NotifyIcon trayIcon = new()
            {
                Text = "Yandex.Music Discord RPC",
                Icon = Resources.YandexMusicDiscordRPCLogo,
                ContextMenuStrip = trayMenu,
                Visible = true
            };

            Application.Run();
        }

        private static void Exit_Click(object? sender, EventArgs e)
        {
            Environment.Exit(0);
        }
    }
}
