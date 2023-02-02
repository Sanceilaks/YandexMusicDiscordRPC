using System.Text.Encodings.Web;
using DiscordRPC;
using DiscordRPC.Logging;

async Task MainAsync()
{
    var rpcClient = new DiscordRpcClient("1070766527147688016");
    rpcClient.Logger = new ConsoleLogger() { Level = LogLevel.Warning };
    rpcClient.OnReady += (sender, message) =>
    {
        Console.WriteLine($"User {message.User} is ready!");
    };
    rpcClient.OnPresenceUpdate += (sender, message) => Console.WriteLine($"Update {message.Presence}");

    rpcClient.Initialize();

    var mediaManger = new WindowsMediaController.MediaManager();
    mediaManger.OnAnyMediaPropertyChanged += (session, properties) =>
    {
        if (!session.ControlSession.SourceAppUserModelId.Contains("Yandex"))
            return;

        Console.ForegroundColor = ConsoleColor.Green;
        Console.WriteLine($"{session.ControlSession.SourceAppUserModelId} - {properties.Title}");

        var url = UrlEncoder.Default.Encode($"{properties.Artist}-{properties.Title}");

        rpcClient.SetPresence(new RichPresence()
        {
            Details = $"{properties.Title}",
            State = $"{properties.Artist}",
            Assets = new Assets()
            {
                LargeImageKey = "og-image",
                LargeImageText = "Яндекс.Музыка",
                SmallImageKey = "og-image",
                SmallImageText = "Яндекс.Музыка"
            },
            Buttons = new[]
            {
                new Button()
                {
                    Label = "Открыть Поиск",
                    Url = $"https://music.yandex.ru/search?text={url}"
                }
            }
        });
    };

    await mediaManger.StartAsync();

    while (mediaManger.IsStarted)
    {
        await Task.Delay(1000);
    }
}

MainAsync().GetAwaiter().GetResult();