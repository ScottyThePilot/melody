use crate::data::Core;

use rand::Rng;
use rand::distributions::WeightedError;
use rand::seq::SliceRandom;
use serenity::model::id::GuildId;
use serenity::model::gateway::ActivityType;
use serenity::gateway::ActivityData;

use std::collections::HashSet;
use std::fmt::{self, Write};



pub(super) fn select(config_activities: &[ConfigActivity], core: &Core) -> Result<ActivityData, ConfigActivityError> {
  let mut rng = rand::thread_rng();
  let activity_data = config_activities.choose_weighted(&mut rng, |v| v.weight)
    .map_err(ConfigActivityError::CannotSelectRandomActivity)?
    .to_activity_data(&mut rng, core)?;
  Ok(activity_data)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigActivity {
  pub weight: u32,
  pub appearance: ConfigActivityAppearance,
  #[serde(flatten)]
  pub variant: ConfigActivityVariant
}

impl ConfigActivity {
  pub fn to_activity_data(&self, rng: &mut impl Rng, core: &Core) -> Result<ActivityData, ConfigActivityError> {
    let text = self.variant.print(rng, core)?;
    Ok(match self.appearance {
      ConfigActivityAppearance::Playing => ActivityData::playing(text),
      ConfigActivityAppearance::Listening => ActivityData::listening(text),
      ConfigActivityAppearance::Watching => ActivityData::watching(text),
      ConfigActivityAppearance::Custom => ActivityData::custom(text),
      ConfigActivityAppearance::Competing => ActivityData::competing(text)
    })
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ConfigActivityVariant {
  #[serde(rename = "global_guild_count")]
  GlobalGuildCount,
  #[serde(rename = "global_user_count")]
  GlobalUserCount,
  #[serde(rename = "text")]
  Text {
    text: String
  },
  #[serde(rename = "concatenate")]
  Concatenate {
    values: Vec<Self>
  },
  #[serde(rename = "select_random")]
  SelectRandom {
    values: Vec<Self>
  },
  #[serde(rename = "select_random_game")]
  SelectRandomGame
}

impl ConfigActivityVariant {
  pub fn print(&self, rng: &mut impl Rng, core: &Core) -> Result<String, ConfigActivityError> {
    let mut buf = String::new();
    self.print_append(&mut buf, rng, core)?;
    Ok(buf)
  }

  fn print_append(&self, buf: &mut String, rng: &mut impl Rng, core: &Core) -> Result<(), ConfigActivityError> {
    Ok(match self {
      Self::GlobalGuildCount => {
        write!(buf, "{}", core.cache.guild_count())?;
      },
      Self::GlobalUserCount => {
        write!(buf, "{}", core.cache.user_count())?;
      },
      Self::Text { text } => {
        buf.push_str(text);
      },
      Self::Concatenate { values } => {
        for value in values {
          value.print_append(buf, rng, core)?;
        };
      },
      Self::SelectRandom { values } => {
        values.choose(rng)
          .ok_or(ConfigActivityError::CannotSelectRandomValue)?
          .print_append(buf, rng, core)?;
      },
      Self::SelectRandomGame => {
        let game = random_game(core)
          .ok_or(ConfigActivityError::CannotSelectRandomGame)?;
        buf.push_str(&game);
      }
    })
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigActivityAppearance {
  #[serde(rename = "playing")] Playing,
  #[serde(rename = "listening")] Listening,
  #[serde(rename = "watching")] Watching,
  #[serde(rename = "custom")] Custom,
  #[serde(rename = "competing")] Competing
}

#[derive(Debug, Error)]
pub enum ConfigActivityError {
  #[error("format error")]
  FormatError(#[from] fmt::Error),
  #[error("failed to select random activity: {0}")]
  CannotSelectRandomActivity(WeightedError),
  #[error("failed to select random value")]
  CannotSelectRandomValue,
  #[error("failed to select random game")]
  CannotSelectRandomGame
}

/// Gets a list of games people are playing in a given guild
fn list_games(core: &Core, guild_id: GuildId) -> HashSet<String> {
  core.cache.guild(guild_id).map_or_else(HashSet::new, |guild| {
    guild.presences.values()
      .filter(|&presence| presence.user.bot != Some(true))
      .flat_map(|presence| presence.activities.iter())
      .filter(|&activity| activity.kind == ActivityType::Playing)
      .map(|activity| activity.name.clone())
      .collect::<HashSet<String>>()
  })
}

fn random_game(core: &Core) -> Option<String> {
  let mut rng = crate::utils::create_rng();
  let games = core.cache.guilds().into_iter()
    .flat_map(|guild_id| list_games(core, guild_id))
    .collect::<Vec<String>>();
  games.choose(&mut rng).cloned()
}
