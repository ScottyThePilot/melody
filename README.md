# Melody

Melody is a Discord bot with a number of features:
- CleverBot integration
- Connect-Four minigame
- YouTube and Twitter feeds
- Server-wide emoji usage stats
- Join roles
- Grantable roles
- Dice rolling

## Terminal

While the bot is running, commands may be issued through the terminal:

- `stop` - Shuts down the bot
- `restart` - Restarts the bot
- `plugin list <guild-id>` - Lists the plugins that are enabled for a given guild
- `plugin enable <plugin> <guild-id>` - Enables a plugin for a given guild
- `plugin disable <plugin> <guild-id>` - Disables a plugin for a given guild
- `feeds respawn-all` - Respawns all feed tasks that may have terminated after too many failures
- `feeds abort-all` - Aborts all feed tasks
- `feeds list-tasks` - Lists all feed tasks and whether they are running

## Plugins

Some commands are assigned a *plugin*, which makes them exclusive and only usable in guilds where their assigned plugin has been enabled.

At the moment, only the `feeds` command has a plugin, the `feed` plugin.

Plugins may be enabled or disabled via the respective terminal commands.

## Configuration

A `config.toml` file must be present in the folder you run Melody from.
Here is an example config:
```toml
# Bot token (required)
token = "your-bot-token-goes-here"
# Bot owner's Discord user ID (required)
owner_id = 356116658398498699
# Accent color shown in help command embeds (optional, defaults to blurple)
accent_color = 0x7289DA
# List of gateway intents the bot should send to the Discord API (optional)
# This can either be a list of intent names, or a number representing a intents bitfield
# Defaults to all non-priveleged intents
intents = [
  "GUILDS",
  "GUILD_MEMBERS",
  "GUILD_BANS",
  "GUILD_EMOJIS_AND_STICKERS",
  "GUILD_MESSAGES",
  "GUILD_MESSAGE_REACTIONS",
  "GUILD_PRESENCES",
  # Without the MESSGE_CONTENT intent, the emoji stats and message chains features will be unavailable
  "MESSAGE_CONTENT"
]

# Settings for YouTube feeds (optional, omit to disable YouTube feeds)
[rss.youtube]
min_delay = 60
max_delay = 7200
frequency_multiplier = 1
# The path that should be used for getting YouTube RSS feeds (optional)
# Defaults to 'www.youtube.com/feeds/videos.xml?channel_id='
base_url = "www.youtube.com/feeds/videos.xml?channel_id="
# The base domain that should be displayed instead of 'www.youtube.com' (optional)
# Use this if you would rather redirect to a privacy frontend like Piped
display_domain = "www.youtube.com"

# Settings for Twitter feeds (optional, omit to disable Twitter feeds)
# This relies on Nitter instances to retrieve RSS feeds of Twitter accounts
# Due to recent events, this may not always be reliable
[rss.twitter]
min_delay = 60
max_delay = 7200
frequency_multiplier = 1
# A list of Nitter instance domains that should be used for fetching RSS feeds (required)
# Note, some Nitter instances (like nitter.net) do not actually support RSS feeds for some reason
nitter_instances = ["unofficialbird.com"]
# The base domain that should be displayed instead of 'twitter.com' (optional)
# Use this if you would rather redirect to a privacy frontend like Nitter or a FixTweet service
display_domain = "vxtwitter.com"
```
