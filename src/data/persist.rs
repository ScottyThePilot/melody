use crate::prelude::*;
use crate::feature::roles::{Granter, JoinRoleFilter};
use crate::feature::feed::{Feed, FeedState};

use serenity::model::id::{GuildId, UserId, RoleId};
use singlefile::container_shared_async::ContainerSharedAsyncWritableLocked;
use singlefile_formats::data::cbor_serde::Cbor;
use tokio::sync::RwLock;

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;

pub type PersistContainer = ContainerSharedAsyncWritableLocked<Persist, Cbor>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Persist {
  pub build_id: u64,
  pub guild_plugins: HashMap<GuildId, HashSet<String>>,
  /// List of users who have been notified that chatbot messages from Melody are from CleverBot.
  pub cleverbot_notified_users: HashSet<UserId>,
  /// List of RSS feeds and their current state.
  pub feeds: HashMap<Feed, FeedState>
}

impl Persist {
  pub async fn create() -> MelodyResult<PersistContainer> {
    fs_err::tokio::create_dir_all("./data/")
      .await.context("failed to create data/guilds/")?;
    let path = PathBuf::from(format!("./data/persist.bin"));
    let container = PersistContainer::create_or_default(path, Cbor)
      .await.context("failed to load data/persist.bin")?;
    trace!("Loaded data/persist.bin");
    Ok(container)
  }

  pub fn swap_build_id(&mut self) -> u64 {
    std::mem::replace(&mut self.build_id, crate::BUILD_ID)
  }

  pub fn cleverbot_notify(&mut self, user_id: UserId) -> bool {
    self.cleverbot_notified_users.insert(user_id)
  }

  pub fn get_guild_plugins_mut(&mut self, id: GuildId) -> &mut HashSet<String> {
    self.guild_plugins.entry(id).or_default()
  }

  pub fn get_guild_plugins(&self, id: GuildId) -> HashSet<String> {
    self.guild_plugins.get(&id).map_or_else(HashSet::new, HashSet::clone)
  }
}

impl Default for Persist {
  fn default() -> Self {
    Persist {
      build_id: 0,
      guild_plugins: HashMap::new(),
      cleverbot_notified_users: HashSet::new(),
      feeds: HashMap::new()
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
    fs_err::tokio::create_dir_all("./data/guilds/")
      .await.context("failed to create data/guilds/")?;
    let mut guilds = HashMap::new();
    let mut read_dir = fs_err::tokio::read_dir("./data/guilds/")
      .await.context("failed to read data/guilds/")?;
    while let Some(entry) = read_dir.next_entry().await.context("failed to read entry in data/guilds/")? {
      let file_type = entry.file_type().await.context("failed to read entry in data/guilds/")?;
      if !file_type.is_file() { continue };
      if let Some(id) = parse_file_name(&entry.file_name()).map(GuildId::new) {
        let persist_guild = PersistGuild::create(id).await?;
        guilds.insert(id, persist_guild);
      };
    };

    Ok(PersistGuilds { guilds })
  }

  pub async fn get(wrapper: &PersistGuildsWrapper, id: GuildId) -> Option<PersistGuildContainer> {
    wrapper.read().await.guilds.get(&id).cloned()
  }

  pub async fn get_default(wrapper: &PersistGuildsWrapper, id: GuildId) -> MelodyResult<PersistGuildContainer> {
    use std::collections::hash_map::Entry;
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

pub type PersistGuildContainer = ContainerSharedAsyncWritableLocked<PersistGuild, Cbor>;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PersistGuild {
  pub connect_four: melody_connect_four::Manager<UserId>,
  #[serde(alias = "emoji_statistics")]
  pub emoji_stats: crate::feature::emoji_stats::EmojiStats,
  pub join_roles: HashMap<RoleId, JoinRoleFilter>,
  pub grant_roles: HashMap<RoleId, HashSet<Granter>>
}

impl PersistGuild {
  pub async fn create(id: GuildId) -> MelodyResult<PersistGuildContainer> {
    let path = PathBuf::from(format!("./data/guilds/{id}.bin"));
    let container = PersistGuildContainer::create_or_default(path, Cbor)
      .await.context(format!("failed to load data/guilds/{id}.bin"))?;
    trace!("Loaded data/guilds/{id}.bin");
    Ok(container)
  }
}

fn parse_file_name(path: &std::ffi::OsStr) -> Option<u64> {
  path.to_str().and_then(|path| path.strip_suffix(".bin")?.parse().ok())
}
