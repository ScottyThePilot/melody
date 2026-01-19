use crate::prelude::*;
use crate::data::Core;

use rand::Rng;
use rand::distr::weighted::Error;
use rand::seq::IndexedRandom;
use serenity::model::id::GuildId;
use serenity::model::gateway::ActivityType;
use serenity::gateway::ActivityData;
use singlefile::container_shared_async::StandardContainerSharedAsync;
use singlefile::manager::StandardManagerOptions;
use singlefile_formats::data::json_serde::Json;

use std::path::PathBuf;
use std::fmt::{self, Write};

const OPTIONS: StandardManagerOptions = StandardManagerOptions::UNLOCKED_WRITABLE;

pub type ActivitiesContainer = StandardContainerSharedAsync<Activities, Json>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Activities {
  pub activities: Vec<Activity>
}

impl Activities {
  #[inline]
  pub async fn create() -> MelodyResult<ActivitiesContainer> {
    let path = PathBuf::from(format!("./data/activities.json"));
    let container = ActivitiesContainer::create_or_default(path, Json::<true>, OPTIONS)
      .await.context("failed to load data/activities.json")?;
    trace!("Loaded data/activities.json");
    Ok(container)
  }

  pub async fn select(&self, core: &Core) -> Result<ActivityData, ActivityError> {
    let emulate_status_modes = core.operate_config(async |config| config.emulate_status_modes).await;

    // TODO: ThreadRng isn't thread safe, making this fully async could have problems with this
    let mut rng = rand::rng();
    let activity_data = self.activities.choose_weighted(&mut rng, |v| v.variant.weight.get())
      .map_err(ActivityError::CannotSelectRandomActivity)?
      .to_activity_data(&mut rng, emulate_status_modes, core)?;
    Ok(activity_data)
  }
}

impl Default for Activities {
  fn default() -> Self {
    let mut watching_users_args = HashMap::new();
    watching_users_args.insert("users".to_owned(), ActivityVariant::GlobalUserCount);

    let watching_users = Activity {
      mode: ActivityMode::Watching,
      variant: ActivityVariantWeighted {
        weight: zu32::MIN,
        variant: ActivityVariant::Format {
          template: "{users} users".to_owned(),
          arguments: watching_users_args
        }
      }
    };

    let mut watching_guilds_args = HashMap::new();
    watching_guilds_args.insert("guilds".to_owned(), ActivityVariant::GlobalGuildCount);

    let watching_guilds = Activity {
      mode: ActivityMode::Watching,
      variant: ActivityVariantWeighted {
        weight: zu32::MIN,
        variant: ActivityVariant::Format {
          template: "{guilds} guilds".to_owned(),
          arguments: watching_guilds_args
        }
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
  pub mode: ActivityMode,
  #[serde(flatten)]
  pub variant: ActivityVariantWeighted
}

impl Activity {
  pub fn to_activity_data(&self, rng: &mut impl Rng, emulate_status_modes: bool, core: &Core) -> Result<ActivityData, ActivityError> {
    if emulate_status_modes {
      self.to_activity_data_emulate_mode(rng, core)
    } else {
      self.to_activity_data_no_emulate_mode(rng, core)
    }
  }

  pub fn to_activity_data_emulate_mode(&self, rng: &mut impl Rng, core: &Core) -> Result<ActivityData, ActivityError> {
    let mut text = String::new();
    if let Some(mode_text) = self.mode.to_str() {
      text.push_str(mode_text);
      text.push(' ');
    };

    self.variant.variant.print_append(&mut text, rng, core)?;
    Ok(ActivityData::custom(text))
  }

  pub fn to_activity_data_no_emulate_mode(&self, rng: &mut impl Rng, core: &Core) -> Result<ActivityData, ActivityError> {
    let text = self.variant.variant.print(rng, core)?;
    Ok(match self.mode {
      ActivityMode::Playing => ActivityData::playing(text),
      ActivityMode::Listening => ActivityData::listening(text),
      ActivityMode::Watching => ActivityData::watching(text),
      ActivityMode::Competing => ActivityData::competing(text),
      ActivityMode::Custom => ActivityData::custom(text)
    })
  }
}

#[inline]
const fn default_weight() -> zu32 {
  zu32::MIN
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityVariantWeighted {
  #[serde(flatten)]
  pub variant: ActivityVariant,
  #[serde(default = "default_weight")]
  pub weight: zu32
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
  #[serde(rename = "template")]
  Format {
    template: String,
    arguments: HashMap<String, Self>
  },
  #[serde(rename = "concatenate")]
  Concatenate {
    values: Vec<Self>
  },
  #[serde(rename = "select_random")]
  SelectRandom {
    values: Vec<ActivityVariantWeighted>
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
      Self::Format { ref template, ref arguments } => {
        assemble_template(buf, rng, core, template, arguments)?;
      },
      Self::SelectRandom { ref values } => {
        values.choose_weighted(rng, |i| i.weight.get())
          .map_err(ActivityError::CannotSelectRandomValue)?
          .variant.print_append(buf, rng, core)?;
      },
      Self::SelectRandomGame => {
        let game = random_game(core)
          .ok_or(ActivityError::CannotSelectRandomGame)?;
        buf.push_str(&game);
      },
      Self::RandomInt { min, max } => {
        write!(buf, "{}", rng.random_range(min..=max))?;
      },
      Self::RandomFloat { min, max, digits } => {
        let value = rng.random_range(min..=max);
        if let Some(digits) = digits {
          write!(buf, "{value:.n$}", n = digits)?;
        } else {
          write!(buf, "{value}", )?;
        };
      }
    })
  }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ActivityMode {
  #[serde(rename = "playing")] Playing,
  #[serde(rename = "listening")] Listening,
  #[serde(rename = "watching")] Watching,
  #[serde(rename = "competing")] Competing,
  #[serde(rename = "custom")] Custom
}

impl ActivityMode {
  pub const fn to_str(self) -> Option<&'static str> {
    match self {
      Self::Playing => Some("Playing"),
      Self::Listening => Some("Listening to"),
      Self::Watching => Some("Watching"),
      Self::Competing => Some("Competing in"),
      Self::Custom => None
    }
  }
}

#[derive(Debug, Error)]
pub enum ActivityError {
  #[error("format error")]
  FormatError(#[from] fmt::Error),
  #[error("failed to assemble template")]
  CannotAssembleTemplate,
  #[error("failed to select random activity: {0}")]
  CannotSelectRandomActivity(Error),
  #[error("failed to select random value")]
  CannotSelectRandomValue(Error),
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
  let mut rng = rand::rng();
  let games = core.cache.guilds().into_iter()
    .flat_map(|guild_id| list_games(core, guild_id))
    .collect::<Vec<String>>();
  games.choose(&mut rng).cloned()
}

fn assemble_template(
  buf: &mut String, rng: &mut impl Rng, core: &Core,
  template: &str, arguments: &HashMap<String, ActivityVariant>
) -> Result<(), ActivityError> {
  let mut iter = template.chars().peekable();

  let mut current_template_argument = String::new();
  let mut inside_template_argument = false;
  while let Some(ch) = iter.next() {
    match ch {
      '{' => {
        if inside_template_argument {
          return Err(ActivityError::CannotAssembleTemplate);
        };

        if let Some('{') = iter.peek() {
          iter.next();
          buf.push('{');
        } else {
          inside_template_argument = true;
        };
      },
      '}' => {
        if !inside_template_argument {
          return Err(ActivityError::CannotAssembleTemplate);
        };

        if let Some('}') = iter.peek() {
          iter.next();
          buf.push('}');
        } else {
          inside_template_argument = false;

          let argument_value = arguments.get(current_template_argument.as_str())
            .ok_or(ActivityError::CannotAssembleTemplate)?;
          argument_value.print_append(buf, rng, core)?;

          current_template_argument.clear();
        };
      },
      ch => if inside_template_argument {
        current_template_argument.push(ch);
      } else {
        buf.push(ch);
      }
    };
  };

  if inside_template_argument {
    return Err(ActivityError::CannotAssembleTemplate);
  };

  Ok(())
}
