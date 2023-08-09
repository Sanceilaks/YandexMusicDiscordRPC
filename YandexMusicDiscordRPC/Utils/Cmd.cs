using System.Diagnostics;

namespace YandexMusicDiscordRPC.Utils
{
    public static class Cmd
    {
        public static void Run(string line) =>
            Process.Start(new ProcessStartInfo("cmd", $"/c {line}") { CreateNoWindow = true });
        public static void Start(string line) =>
            Run($"start {line}");
    }
}
