use crate::MelodyResult;
use crate::utils::Contextualize;
use super::Toml;

use serenity::model::id::UserId;
use serenity::utils::Color;
use singlefile::container_shared_async::ContainerAsyncReadonly;

use std::path::PathBuf;

pub type ConfigContainer = ContainerAsyncReadonly<Config, Toml>;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
  pub token: String,
  pub owner_id: UserId,
  #[serde(default)]
  pub accent_color: Option<Color>
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
