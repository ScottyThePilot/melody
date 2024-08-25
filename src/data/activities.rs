use crate::MelodyResult;
use crate::data::Core;
use crate::utils::Contextualize;

use rand::Rng;
use rand::distributions::WeightedError;
use rand::seq::SliceRandom;
use serenity::model::id::GuildId;
use serenity::model::gateway::ActivityType;
use serenity::gateway::ActivityData;
use singlefile::container_shared_async::ContainerSharedAsyncReadonly;
use singlefile_formats::json_serde::Json;

use std::collections::HashSet;
use std::path::PathBuf;
use std::fmt::{self, Write};

pub type ActivitiesContainer = ContainerSharedAsyncReadonly<Activities, Json>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Activities {
  pub activities: Vec<Activity>
}

impl Activities {
  #[inline]
  pub async fn create() -> MelodyResult<ActivitiesContainer> {
    let path = PathBuf::from(format!("./data/activities.json"));
    let container = ActivitiesContainer::create_or_default(path, Json::<true>)
      .await.context("failed to load data/activities.json")?;
    trace!("Loaded data/activities.json");
    Ok(container)
  }

  pub fn select(&self, core: &Core) -> Result<ActivityData, ActivityError> {
    let mut rng = rand::thread_rng();
    let activity_data = self.activities.choose_weighted(&mut rng, |v| v.weight)
      .map_err(ActivityError::CannotSelectRandomActivity)?
      .to_activity_data(&mut rng, core)?;
    Ok(activity_data)
  }
}

impl Default for Activities {
  fn default() -> Self {
    let watching_users = Activity {
      mode: ActivityMode::Watching,
      weight: 1,
      variant: ActivityVariant::Concatenate {
        values: vec![
          ActivityVariant::GlobalUserCount,
          ActivityVariant::Text { text: " users".to_owned() }
        ]
      }
    };

    let watching_guilds = Activity {
      mode: ActivityMode::Watching,
      weight: 1,
      variant: ActivityVariant::Concatenate {
        values: vec![
          ActivityVariant::GlobalGuildCount,
          ActivityVariant::Text { text: " guilds".to_owned() }
        ]
      }
    };

    Activities {
      activities: vec![
        watching_users,
        watching_guilds
      ]
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
  pub weight: u32,
  pub mode: ActivityMode,
  #[serde(flatten)]
  pub variant: ActivityVariant
}

impl Activity {
  pub fn to_activity_data(&self, rng: &mut impl Rng, core: &Core) -> Result<ActivityData, ActivityError> {
    let text = self.variant.print(rng, core)?;
    Ok(match self.mode {
      ActivityMode::Playing => ActivityData::playing(text),
      ActivityMode::Listening => ActivityData::listening(text),
      ActivityMode::Watching => ActivityData::watching(text),
      ActivityMode::Custom => ActivityData::custom(text),
      ActivityMode::Competing => ActivityData::competing(text)
    })
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ActivityVariant {
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
  SelectRandomGame,
  #[serde(rename = "random_int")]
  RandomInt {
    min: i64,
    max: i64
  },
  #[serde(rename = "random_float")]
  RandomFloat {
    min: f64,
    max: f64,
    #[serde(default)]
    digits: Option<usize>
  }
}

impl ActivityVariant {
  pub fn print(&self, rng: &mut impl Rng, core: &Core) -> Result<String, ActivityError> {
    let mut buf = String::new();
    self.print_append(&mut buf, rng, core)?;
    Ok(buf)
  }

  fn print_append(&self, buf: &mut String, rng: &mut impl Rng, core: &Core) -> Result<(), ActivityError> {
    Ok(match *self {
      Self::GlobalGuildCount => {
        write!(buf, "{}", core.cache.guild_count())?;
      },
      Self::GlobalUserCount => {
        write!(buf, "{}", core.cache.user_count())?;
      },
      Self::Text { ref text } => {
        buf.push_str(text);
      },
      Self::Concatenate { ref values } => {
        for value in values {
          value.print_append(buf, rng, core)?;
        };
      },
      Self::SelectRandom { ref values } => {
        values.choose(rng)
          .ok_or(ActivityError::CannotSelectRandomValue)?
          .print_append(buf, rng, core)?;
      },
      Self::SelectRandomGame => {
        let game = random_game(core)
          .ok_or(ActivityError::CannotSelectRandomGame)?;
        buf.push_str(&game);
      },
      Self::RandomInt { min, max } => {
        write!(buf, "{}", rng.gen_range(min..=max))?;
      },
      Self::RandomFloat { min, max, digits } => {
        let value = rng.gen_range(min..=max);
        if let Some(digits) = digits {
          write!(buf, "{value:.n$}", n = digits)?;
        } else {
          write!(buf, "{value}", )?;
        };
      }
    })
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActivityMode {
  #[serde(rename = "playing")] Playing,
  #[serde(rename = "listening")] Listening,
  #[serde(rename = "watching")] Watching,
  #[serde(rename = "custom")] Custom,
  #[serde(rename = "competing")] Competing
}

#[derive(Debug, Error)]
pub enum ActivityError {
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
