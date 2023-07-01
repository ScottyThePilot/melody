use crate::MelodyResult;
use crate::utils::Contextualize;
use super::Toml;

use serde::de::{Deserialize, Deserializer, Unexpected};
use serenity::model::id::UserId;
use serenity::model::gateway::GatewayIntents;
use serenity::utils::Color;
use singlefile::container_shared_async::ContainerAsyncReadonly;

use std::path::PathBuf;

pub type ConfigContainer = ContainerAsyncReadonly<Config, Toml>;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
  pub token: String,
  pub owner_id: UserId,
  #[serde(default)]
  pub accent_color: Option<Color>,
  // defaults to `GatewayIntents::non_privileged`
  #[serde(default, deserialize_with = "deserialize_intents")]
  pub intents: GatewayIntents
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

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum ConfigGatewayIntents {
  Name(ConfigGatewayIntentsIdentifier),
  NameList(Vec<ConfigGatewayIntentsIdentifier>),
  GatewayIntents(GatewayIntents)
}

fn deserialize_intents<'de, D: Deserializer<'de>>(deserializer: D) -> Result<GatewayIntents, D::Error> {
  ConfigGatewayIntents::deserialize(deserializer).map(|intents| match intents {
    ConfigGatewayIntents::Name(identifier) => identifier.intents,
    ConfigGatewayIntents::NameList(identifier_list) => intents_from_list(identifier_list),
    ConfigGatewayIntents::GatewayIntents(intents) => intents
  })
}

fn intents_from_list(list: Vec<ConfigGatewayIntentsIdentifier>) -> GatewayIntents {
  list.into_iter().map(|identifier| identifier.intents).collect()
}

#[derive(Debug, Clone, Copy)]
struct ConfigGatewayIntentsIdentifier {
  intents: GatewayIntents
}

impl<'de> Deserialize<'de> for ConfigGatewayIntentsIdentifier {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where D: Deserializer<'de> {
    let identifier = String::deserialize(deserializer)?.to_ascii_lowercase();
    let intents = match identifier.as_str() {
      "guilds" => GatewayIntents::GUILDS,
      "guild_members" => GatewayIntents::GUILD_MEMBERS,
      "guild_bans" => GatewayIntents::GUILD_BANS,
      "guild_emojis_and_stickers" => GatewayIntents::GUILD_EMOJIS_AND_STICKERS,
      "guild_integrations" => GatewayIntents::GUILD_INTEGRATIONS,
      "guild_webhooks" => GatewayIntents::GUILD_WEBHOOKS,
      "guild_invites" => GatewayIntents::GUILD_INVITES,
      "guild_voice_states" => GatewayIntents::GUILD_VOICE_STATES,
      "guild_presences" => GatewayIntents::GUILD_PRESENCES,
      "guild_messages" => GatewayIntents::GUILD_MESSAGES,
      "guild_message_reactions" => GatewayIntents::GUILD_MESSAGE_REACTIONS,
      "guild_message_typing" => GatewayIntents::GUILD_MESSAGE_TYPING,
      "direct_messages" => GatewayIntents::DIRECT_MESSAGES,
      "direct_message_reactions" => GatewayIntents::DIRECT_MESSAGE_REACTIONS,
      "direct_message_typing" => GatewayIntents::DIRECT_MESSAGE_TYPING,
      "message_content" => GatewayIntents::MESSAGE_CONTENT,
      "guild_scheduled_events" => GatewayIntents::GUILD_SCHEDULED_EVENTS,
      "auto_moderation_configuration" => GatewayIntents::AUTO_MODERATION_CONFIGURATION,
      "auto_moderation_execution" => GatewayIntents::AUTO_MODERATION_EXECUTION,
      "non_privileged" => GatewayIntents::non_privileged(),
      "privileged" => GatewayIntents::privileged(),
      "all" => GatewayIntents::all(),
      identifier => return Err({
        serde::de::Error::invalid_value(Unexpected::Str(identifier), &"gateway intents identifier")
      })
    };

    Ok(ConfigGatewayIntentsIdentifier { intents })
  }
}

// GUILDS
// GUILD_MEMBERS
// GUILD_BANS
// GUILD_EMOJIS_AND_STICKERS
// GUILD_MESSAGES
// GUILD_MESSAGE_REACTIONS
// GUILD_PRESENCES
// MESSAGE_CONTENT
