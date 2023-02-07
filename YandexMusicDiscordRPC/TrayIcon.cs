using YandexMusicDiscordRPC.Properties;
using YandexMusicDiscordRPC.Utils;

namespace YandexMusicDiscordRPC
{
    public class TrayIcon
    {
        public static void Run()
        {
            ContextMenuStrip trayMenu = new()
            {
                Items = {
                    new ToolStripMenuItem("About", IconExtractor.Extract("imageres.dll", 76, true)!.ToBitmap(), About_Click),
                    new ToolStripMenuItem("Exit", IconExtractor.Extract("shell32.dll", 131, true)!.ToBitmap(), Exit_Click),
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

        private static void About_Click(object? sender, EventArgs e)
        {
            Cmd.Start("https://github.com/Sanceilaks/YandexMusicDiscordRPC");
        }

        private static void Exit_Click(object? sender, EventArgs e)
        {
            Environment.Exit(0);
        }
    }
}
