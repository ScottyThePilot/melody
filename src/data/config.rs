use crate::prelude::*;

use rand::seq::IndexedRandom;
use serde::de::{Deserialize, Deserializer, Unexpected};
use serenity::model::gateway::GatewayIntents;
use serenity::model::colour::Color;
use singlefile::container_shared_async::ContainerSharedAsyncReadonly;
use singlefile_formats::data::toml_serde::Toml;

use std::path::PathBuf;
use std::sync::Arc;
use std::str::FromStr;
use std::time::Duration;

pub type ConfigContainer = ContainerSharedAsyncReadonly<Config, Toml>;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
  /// Bot token (required)
  pub token: String,
  /// Accent color shown in help command embeds (optional)
  #[serde(default)]
  pub accent_color: Option<Color>,
  /// Messages will not be sent at a higher interval than this.
  #[serde(default = "default_cleverbot_ratelimit", deserialize_with = "deserialize_duration")]
  pub cleverbot_ratelimit: Duration,
  /// The list of gateway intents the bot should send to the Discord API.
  /// This can either be a list of intent names, or a number representing an intents bitfield.
  /// Defaults to [`GatewayIntents::non_privileged`].
  #[serde(default = "GatewayIntents::non_privileged", deserialize_with = "deserialize_intents")]
  pub intents: GatewayIntents,
  #[serde(default)]
  pub rss: ConfigRss,
  #[serde(default)]
  pub music_player: Option<ConfigMusicPlayer>
}

impl Config {
  #[inline]
  pub async fn create() -> MelodyResult<ConfigContainer> {
    let path = PathBuf::from(format!("./config.toml"));
    let container = ConfigContainer::create_or_default(path, Toml)
      .await.context("failed to load config.toml")?;
    trace!("Loaded config.toml");
    Ok(container)
  }
}

fn default_cleverbot_ratelimit() -> Duration {
  Duration::from_secs_f64(5.0)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigMusicPlayer {
  /// The path to the `yt-dlp` executable.
  #[serde(alias = "ytdlp_path")]
  pub yt_dlp_path: PathBuf
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ConfigRss {
  pub youtube: Option<Arc<ConfigRssYouTube>>,
  pub twitter: Option<Arc<ConfigRssTwitter>>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigRssYouTube {
  /// The base domain and URL path that should be used for getting YouTube RSS feeds.
  ///
  /// Defaults to `www.youtube.com/feeds/videos.xml?channel_id=`.
  #[serde(default = "default_youtube_base_url")]
  pub base_url: String,
  /// The interval between fetches for each individual registered RSS feed.
  #[serde(deserialize_with = "deserialize_duration")]
  pub interval: Duration,
  /// The base domain that should be used when displaying YouTube URLs.
  /// This domain should respond to URLs the same as YouTube itself.
  ///
  /// Defaults to `www.youtube.com`.
  #[serde(default = "default_youtube_display_domain")]
  pub display_domain: String
}

impl ConfigRssYouTube {
  pub fn get_url(&self, channel: &str) -> String {
    let base_url = self.base_url.trim_start_matches("https://");
    format!("https://{base_url}{channel}")
  }
}

fn default_youtube_base_url() -> String {
  "www.youtube.com/feeds/videos.xml?channel_id=".to_owned()
}

fn default_youtube_display_domain() -> String {
  "www.youtube.com".to_owned()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigRssTwitter {
  /// A list of Nitter domains that should be used for fetching RSS feeds.
  /// Note, some Nitter instances don't seem to support RSS feeds or are broken.
  #[serde(deserialize_with = "deserialize_at_least_one")]
  pub nitter_instances: Vec<String>,
  /// The interval between fetches for each individual registered RSS feed.
  #[serde(deserialize_with = "deserialize_duration")]
  pub interval: Duration,
  /// The base domain that should be used when displaying Twitter URLs.
  /// This domain should respond to URLs the same as Twitter itself.
  ///
  /// Defaults to `twitter.com` (consider using `vxtwitter.com`).
  #[serde(default = "default_twitter_display_domain")]
  pub display_domain: String
}

impl ConfigRssTwitter {
  pub fn get_url(&self, handle: &str) -> String {
    let mut rng = rand::rng();
    self.nitter_instances.choose(&mut rng).map(|domain| {
      let domain = domain.trim_start_matches("https://").trim_end_matches('/');
      format!("https://{domain}/{handle}/rss")
    }).expect("invalid nitter instances list")
  }
}

fn default_twitter_display_domain() -> String {
  "twitter.com".to_owned()
}

fn deserialize_duration<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Duration, D::Error> {
  f64::deserialize(deserializer).map(Duration::from_secs_f64)
}

fn deserialize_at_least_one<'de, T, D>(deserializer: D) -> Result<Vec<T>, D::Error>
where T: Deserialize<'de>, D: Deserializer<'de> {
  <Vec<T>>::deserialize(deserializer).and_then(|value| match value.is_empty() {
    true => Err(serde::de::Error::invalid_length(0, &"a sequence of at least length 1")),
    false => Ok(value)
  })
}

fn deserialize_intents<'de, D: Deserializer<'de>>(deserializer: D) -> Result<GatewayIntents, D::Error> {
  #[derive(Debug, Clone, Deserialize)]
  #[serde(untagged)]
  enum ConfigGatewayIntents {
    Name(ConfigGatewayIntentsIdentifier),
    NameList(Vec<ConfigGatewayIntentsIdentifier>),
    GatewayIntents(GatewayIntents)
  }

  #[derive(Debug, Clone, Copy)]
  struct ConfigGatewayIntentsIdentifier {
    intents: GatewayIntents
  }

  impl FromStr for ConfigGatewayIntentsIdentifier {
    type Err = ();

    fn from_str(string: &str) -> Result<Self, Self::Err> {
      GatewayIntents::from_name(&string)
        .or_else(|| match string {
          "NON_PRIVILEGED" => Some(GatewayIntents::non_privileged()),
          "PRIVILEGED" => Some(GatewayIntents::privileged()),
          "ALL" => Some(GatewayIntents::all()),
          _ => None
        })
        .map(|intents| ConfigGatewayIntentsIdentifier { intents })
        .ok_or(())
    }
  }

  impl<'de> Deserialize<'de> for ConfigGatewayIntentsIdentifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
      let mut identifier = String::deserialize(deserializer)?;
      identifier.make_ascii_uppercase();
      identifier.parse::<ConfigGatewayIntentsIdentifier>()
        .map_err(|()| serde::de::Error::invalid_value(
          Unexpected::Str(identifier.as_str()),
          &"gateway intents identifier"
        ))
    }
  }

  fn intents_from_list(list: Vec<ConfigGatewayIntentsIdentifier>) -> GatewayIntents {
    list.into_iter().map(|identifier| identifier.intents).collect()
  }

  ConfigGatewayIntents::deserialize(deserializer).map(|intents| match intents {
    ConfigGatewayIntents::Name(identifier) => identifier.intents,
    ConfigGatewayIntents::NameList(identifier_list) => intents_from_list(identifier_list),
    ConfigGatewayIntents::GatewayIntents(intents) => intents
  })
}



// GUILDS
// GUILD_MEMBERS
// GUILD_BANS
// GUILD_EMOJIS_AND_STICKERS
// GUILD_MESSAGES
// GUILD_MESSAGE_REACTIONS
// GUILD_PRESENCES
// MESSAGE_CONTENT
