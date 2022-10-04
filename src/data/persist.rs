use super::Bincode;

use serenity::model::id::GuildId;
use singlefile::Error as FileError;
use singlefile::container_tokio::ContainerAsyncWritableLocked;
use tokio::sync::RwLock;

use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry;
use std::path::PathBuf;
use std::sync::Arc;

pub type PersistContainer = ContainerAsyncWritableLocked<Persist, Bincode>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Persist {
  build_id: u64,
  guild_plugins: HashMap<u64, HashSet<String>>
}

impl Persist {
  pub async fn create() -> Result<PersistContainer, FileError> {
    tokio::fs::create_dir_all("./data/").await?;
    let path = PathBuf::from(format!("./data/persist.bin"));
    PersistContainer::create_or_default(path, Bincode).await
  }

  pub async fn swap_build_id(container: &PersistContainer) -> u64 {
    std::mem::replace(&mut container.access_mut().await.build_id, crate::build_id::get())
  }

  pub async fn get_guild_plugins(container: &PersistContainer, id: GuildId) -> HashSet<String> {
    Self::guild_plugins_mut(container, id, |plugins| plugins.clone()).await
  }

  pub async fn add_guild_plugin(container: &PersistContainer, id: GuildId, plugin: impl Into<String>) -> bool {
    Self::guild_plugins_mut(container, id, |plugins| plugins.insert(plugin.into())).await
  }

  pub async fn remove_guild_plugin(container: &PersistContainer, id: GuildId, plugin: impl AsRef<str>) -> bool {
    Self::guild_plugins_mut(container, id, |plugins| plugins.remove(plugin.as_ref())).await
  }

  async fn guild_plugins_mut<F, R>(container: &PersistContainer, id: GuildId, f: F) -> R
  where F: FnOnce(&mut HashSet<String>) -> R {
    f(container.access_mut().await.guild_plugins.entry(id.into()).or_default())
  }
}

impl Default for Persist {
  fn default() -> Self {
    Persist {
      build_id: 0,
      guild_plugins: HashMap::new()
    }
  }
}

pub type PersistGuildsWrapper = Arc<RwLock<PersistGuilds>>;

#[derive(Debug, Clone, Default)]
pub struct PersistGuilds {
  guilds: HashMap<GuildId, PersistGuildContainer>
}

impl PersistGuilds {
  #[inline]
  pub fn create() -> PersistGuildsWrapper {
    Arc::new(RwLock::new(Self::default()))
  }

  pub async fn get(wrapper: PersistGuildsWrapper, id: GuildId) -> Option<PersistGuildContainer> {
    wrapper.read().await.guilds.get(&id).cloned()
  }

  pub async fn get_default(wrapper: PersistGuildsWrapper, id: GuildId) -> Result<PersistGuildContainer, FileError> {
    match wrapper.write().await.guilds.entry(id) {
      Entry::Occupied(occupied) => Ok(occupied.get().clone()),
      Entry::Vacant(vacant) => Ok(vacant.insert(PersistGuild::create(id).await?).clone())
    }
  }
}

pub type PersistGuildContainer = ContainerAsyncWritableLocked<PersistGuild, Bincode>;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PersistGuild {
  pub connect_four: crate::feature::connect_four::ConnectFourManager
}

impl PersistGuild {
  pub async fn create(id: GuildId) -> Result<PersistGuildContainer, FileError> {
    tokio::fs::create_dir_all("./data/guilds/").await?;
    let path = PathBuf::from(format!("./data/guilds/{id}.bin"));
    PersistGuildContainer::create_or_default(path, Bincode).await
  }
}
