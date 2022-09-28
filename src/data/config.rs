use super::Toml;

use serenity::model::id::{GuildId, UserId};
use serenity::utils::Color;
use singlefile::Error as FileError;
use singlefile::container_tokio::ContainerAsyncReadonly;

use std::path::PathBuf;

pub type ConfigContainer = ContainerAsyncReadonly<Config, Toml>;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
  pub token: String,
  pub owner_id: UserId,
  #[serde(default)]
  pub test_guild: Option<GuildId>,
  #[serde(default)]
  pub accent_color: Option<Color>
}

impl Config {
  #[inline]
  pub async fn create() -> Result<ConfigContainer, FileError> {
    let path = PathBuf::from(format!("./config.toml"));
    ConfigContainer::create_or_default(path, Toml).await
  }
}
