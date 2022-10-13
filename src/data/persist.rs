use crate::MelodyResult;
use crate::utils::Contextualize;
use super::Cbor;

use serenity::model::id::GuildId;
use singlefile::container_shared_async::ContainerAsyncWritableLocked;
use tokio::sync::RwLock;

use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry;
use std::path::PathBuf;
use std::sync::Arc;

pub type PersistContainer = ContainerAsyncWritableLocked<Persist, Cbor>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Persist {
  build_id: u64,
  guild_plugins: HashMap<u64, HashSet<String>>
}

impl Persist {
  pub async fn create() -> MelodyResult<PersistContainer> {
    tokio::fs::create_dir_all("./data/")
      .await.context("failed to create data/guilds/")?;
    let path = PathBuf::from(format!("./data/persist.bin"));
    let container = PersistContainer::create_or_default(path, Cbor)
      .await.context("failed to load data/persist.bin")?;
    trace!("loaded data/persist.bin");
    Ok(container)
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
  pub async fn create() -> MelodyResult<PersistGuilds> {
    let mut guilds = HashMap::new();
    let mut read_dir = tokio::fs::read_dir("./data/guilds/")
      .await.context("failed to read dir")?;
    while let Some(entry) = read_dir.next_entry().await.context("failed to read dir")? {
      let file_type = entry.file_type().await.context("failed to read dir")?;
      if !file_type.is_dir() { continue };
      if let Some(id) = parse_file_name(&entry.file_name()).map(GuildId) {
        let persist_guild = PersistGuild::create(id).await?;
        guilds.insert(id, persist_guild);
      };
    };

    Ok(PersistGuilds { guilds })
  }

  pub async fn get(wrapper: PersistGuildsWrapper, id: GuildId) -> Option<PersistGuildContainer> {
    wrapper.read().await.guilds.get(&id).cloned()
  }

  pub async fn get_default(wrapper: PersistGuildsWrapper, id: GuildId) -> MelodyResult<PersistGuildContainer> {
    match wrapper.write().await.guilds.entry(id) {
      Entry::Occupied(occupied) => Ok(occupied.get().clone()),
      Entry::Vacant(vacant) => Ok(vacant.insert(PersistGuild::create(id).await?).clone())
    }
  }
}

impl From<PersistGuilds> for PersistGuildsWrapper {
  fn from(persist_guilds: PersistGuilds) -> Self {
    Arc::new(RwLock::new(persist_guilds))
  }
}

pub type PersistGuildContainer = ContainerAsyncWritableLocked<PersistGuild, Cbor>;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PersistGuild {
  pub connect_four: crate::feature::connect_four::Manager
}

impl PersistGuild {
  pub async fn create(id: GuildId) -> MelodyResult<PersistGuildContainer> {
    tokio::fs::create_dir_all("./data/guilds/")
      .await.context("failed to create data/guilds/")?;
    let path = PathBuf::from(format!("./data/guilds/{id}.bin"));
    let container = PersistGuildContainer::create_or_default(path, Cbor)
      .await.context(format!("failed to load data/guilds/{id}.bin"))?;
    trace!("loaded data/guilds/{id}.bin");
    Ok(container)
  }
}

fn parse_file_name(path: &std::ffi::OsStr) -> Option<u64> {
  path.to_str().and_then(|path| path.parse().ok())
}
