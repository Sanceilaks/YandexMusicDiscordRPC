using DiscordRPC;
using DiscordRPC.Logging;
using System.Text.Encodings.Web;
using Windows.Media.Control;
using YandexMusicDiscordRPC;

async Task MainAsync()
{
    Console.ForegroundColor = ConsoleColor.Green;

    var rpcClient = new DiscordRpcClient("1071095852359233556")
    {
        Logger = new ConsoleLogger() { Level = LogLevel.Warning }
    };
    rpcClient.OnReady += (sender, message) =>
    {
        Console.WriteLine($"User {message.User} is ready!");
    };
    rpcClient.OnPresenceUpdate += (sender, message) => Console.WriteLine($"Updated RPC!");

    rpcClient.Initialize();

    //new Thread(TrayIcon.Run).Start();
    _ = Task.Run(() => TrayIcon.Run());

    var mediaManger = new WindowsMediaController.MediaManager();
    mediaManger.OnAnyMediaPropertyChanged += (session, properties) =>
    {
        if (!session.ControlSession.SourceAppUserModelId.Contains("Yandex.Music"))
            return;

        var isPlaying = session.ControlSession.GetPlaybackInfo().PlaybackStatus.Equals(GlobalSystemMediaTransportControlsSessionPlaybackStatus.Playing);
        var url = UrlEncoder.Default.Encode($"{properties.Artist} - {properties.Title}");

        Console.WriteLine($"{session.ControlSession.SourceAppUserModelId} property update: {properties.Artist} - {properties.Title} ({properties.AlbumTitle}) ");

        rpcClient.SetPresence(new RichPresence()
        {
            Details = $"🎵 {properties.Title}",
            State = $"👤 {properties.Artist}",
            Assets = new Assets()
            {
                LargeImageKey = "logo",
                LargeImageText = $"💿 {properties.AlbumTitle}",
                SmallImageKey = isPlaying ? "playing" : "paused",
                SmallImageText = isPlaying ? "▶️ Playing" : "⏸️ Paused"
            },
            Buttons = new[]
            {
                new DiscordRPC.Button()
                {
                    Label = "Open search",
                    Url = $"https://music.yandex.ru/search?text={url}"
                }
            }
        });
    };

    mediaManger.OnAnyPlaybackStateChanged += (session, playbackInfo) =>
    {
        if (!session.ControlSession.SourceAppUserModelId.Contains("Yandex.Music"))
            return;

        var isPlaying = playbackInfo.PlaybackStatus.Equals(GlobalSystemMediaTransportControlsSessionPlaybackStatus.Playing);

        Console.WriteLine($"{session.ControlSession.SourceAppUserModelId} state update: {playbackInfo.PlaybackStatus}");

        rpcClient.UpdateSmallAsset(isPlaying ? "playing" : "paused", isPlaying ? "▶️ Playing" : "⏸️ Paused");

    };

    mediaManger.OnAnySessionClosed += (session) =>
    {
        if (!session.ControlSession.SourceAppUserModelId.Contains("Yandex.Music"))
            return;

        Console.WriteLine($"{session.ControlSession.SourceAppUserModelId} session ended!");

        rpcClient.ClearPresence();
    };

    await mediaManger.StartAsync();

    while (mediaManger.IsStarted)
    {
        await Task.Delay(1000);
    }

}

MainAsync().GetAwaiter().GetResult();