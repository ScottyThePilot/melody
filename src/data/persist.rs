use crate::MelodyResult;
use crate::utils::Contextualize;
use super::Cbor;

use std::collections::{HashMap, HashSet};
use serenity::model::guild::Emoji;
use serenity::model::id::{EmojiId, GuildId, UserId};
use singlefile::container_shared_async::ContainerAsyncWritableLocked;
use tokio::sync::RwLock;

use std::collections::hash_map::Entry;
use std::path::PathBuf;
use std::sync::Arc;

pub type PersistContainer = ContainerAsyncWritableLocked<Persist, Cbor>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Persist {
  pub build_id: u64,
  pub guild_plugins: HashMap<GuildId, HashSet<String>>,
  /// List of users who have been notified that chatbot messages from Melody are from CleverBot.
  pub cleverbot_notified_users: HashSet<UserId>
}

impl Persist {
  pub async fn create() -> MelodyResult<PersistContainer> {
    tokio::fs::create_dir_all("./data/")
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
      cleverbot_notified_users: HashSet::new()
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
      if !file_type.is_file() { continue };
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
  pub connect_four: crate::feature::connect_four::Manager,
  /// List of emojis and the number of times they've been used in this server.
  pub emoji_statistics: HashMap<EmojiId, usize>,
}

impl PersistGuild {
  pub async fn create(id: GuildId) -> MelodyResult<PersistGuildContainer> {
    tokio::fs::create_dir_all("./data/guilds/")
      .await.context("failed to create data/guilds/")?;
    let path = PathBuf::from(format!("./data/guilds/{id}.bin"));
    let container = PersistGuildContainer::create_or_default(path, Cbor)
      .await.context(format!("failed to load data/guilds/{id}.bin"))?;
    trace!("Loaded data/guilds/{id}.bin");
    Ok(container)
  }

  pub fn increment_emoji_uses(&mut self, emoji: EmojiId) {
    *self.emoji_statistics.entry(emoji).or_default() += 1;
  }

  pub fn decrement_emoji_uses(&mut self, emoji: EmojiId) {
    if let Some(value) = self.emoji_statistics.get_mut(&emoji) {
      *value = value.saturating_sub(1);
    };
  }

  pub fn get_emoji_uses<'a, F>(&self, mut f: F) -> Vec<(EmojiId, bool, usize)>
  where F: FnMut(EmojiId) -> Option<&'a Emoji> {
    let mut emoji_statistics = self.emoji_statistics.iter()
      .filter_map(|(&id, &c)| (c > 0).then_some((id, c)))
      .filter_map(|(id, c)| f(id).map(|emoji| (emoji.id, emoji.animated, c)))
      .collect::<Vec<(EmojiId, bool, usize)>>();
    emoji_statistics.sort_unstable_by(|a, b| Ord::cmp(&a.2, &b.2).reverse());
    emoji_statistics
  }
}

fn parse_file_name(path: &std::ffi::OsStr) -> Option<u64> {
  path.to_str().and_then(|path| path.strip_suffix(".bin")?.parse().ok())
}
