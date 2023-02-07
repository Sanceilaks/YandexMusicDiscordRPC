using System.Diagnostics;

namespace YandexMusicDiscordRPC.Utils
{
    public static class Cmd
    {
        public static void Start(string line) =>
            Process.Start(new ProcessStartInfo("cmd", $"/c start {line}") { CreateNoWindow = true });
        public static void Run(string line) =>
            Process.Start(new ProcessStartInfo("cmd", $"/c {line}") { CreateNoWindow = true });
    }
}
