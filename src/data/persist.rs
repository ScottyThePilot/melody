use super::Bincode;

use serenity::model::id::GuildId;
use singlefile::Error as FileError;
use singlefile::container_tokio::ContainerAsyncWritableLocked;
use tokio::sync::RwLock;

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::path::PathBuf;
use std::sync::Arc;

pub type PersistContainer = ContainerAsyncWritableLocked<Persist, Bincode>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Persist {
  build_id: u64
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
}

impl Default for Persist {
  fn default() -> Self {
    Persist {
      build_id: 0
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
    wrapper.read_owned().await.guilds.get(&id).cloned()
  }

  pub async fn get_default(wrapper: PersistGuildsWrapper, id: GuildId) -> Result<PersistGuildContainer, FileError> {
    match wrapper.write_owned().await.guilds.entry(id) {
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
