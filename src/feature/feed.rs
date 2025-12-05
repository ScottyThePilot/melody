use crate::prelude::*;
use crate::data::{Core, ConfigRss, ConfigRssTwitter, ConfigRssYouTube};

use ahash::{AHashMap, AHashSet};
use chrono::{DateTime, Utc};
use feed_machine::handle::{Context, Model, HandleWithContext};
use feed_machine::model::twitter::TwitterPost;
use feed_machine::model::youtube::YouTubeVideo;
use reqwest::Client;
use serenity::model::id::{ChannelId, GuildId};
use tokio::sync::{RwLock, RwLockWriteGuard, RwLockMappedWriteGuard};
use tokio::time::sleep;
use url::Url;

use std::fmt;
use std::hash::{Hash, Hasher};
use std::time::Duration;



#[derive(Debug)]
pub struct FeedManager {
  context: FeedContext,
  handle_youtube: RwLock<Option<HandleYouTube>>,
  handle_twitter: RwLock<Option<HandleTwitter>>
}

impl FeedManager {
  pub fn new(core: Core, client: Client, config: &ConfigRss) -> Self {
    let context = FeedContext { core, client };

    let handle_youtube = config.youtube.clone()
      .map(|config| HandleYouTube::new(FeedModelYouTube { config }, context.clone()));
    let handle_twitter = config.twitter.clone()
      .map(|config| HandleTwitter::new(FeedModelTwitter { config }, context.clone()));

    FeedManager {
      context,
      handle_youtube: RwLock::new(handle_youtube),
      handle_twitter: RwLock::new(handle_twitter)
    }
  }

  pub async fn spawn_feeds_from_persist(&self) -> MelodyResult<()> {
    let (guard_youtube, guard_twitter) = tokio::join!(self.write_youtube(), self.write_twitter());

    let mut feed_identifiers_youtube = Vec::new();
    let mut feed_identifiers_twitter = Vec::new();
    self.context.core.operate_persist(async |persist| {
      if guard_youtube.is_some() {
        feed_identifiers_youtube.extend(persist.feed_states.youtube.keys().cloned());
      };

      if guard_twitter.is_some() {
        feed_identifiers_twitter.extend(persist.feed_states.twitter.keys().cloned());
      };
    }).await;

    tokio::join!(
      async {
        if let Some(guard_youtube) = guard_youtube {
          guard_youtube.replace_queue(feed_identifiers_youtube).await;
        };
      },
      async {
        if let Some(guard_twitter) = guard_twitter {
          guard_twitter.replace_queue(feed_identifiers_twitter).await;
        };
      }
    );

    Ok(())
  }

  pub async fn register_feed(&self, feed_identifier: FeedIdentifier, guild_id: GuildId, channel_id: ChannelId) -> MelodyResult<RegisterFeedResult> {
    match feed_identifier {
      FeedIdentifier::YouTube(feed_identifier) => self.register_feed_youtube(feed_identifier, guild_id, channel_id).await,
      FeedIdentifier::Twitter(feed_identifier) => self.register_feed_twitter(feed_identifier, guild_id, channel_id).await
    }
  }

  pub async fn register_feed_youtube(&self, feed_identifier: FeedIdentifierYouTube, guild_id: GuildId, channel_id: ChannelId) -> MelodyResult<RegisterFeedResult> {
    let Some(guard) = self.write_youtube().await else { return Ok(RegisterFeedResult::FeedNotEnabled) };
    let result = self.register_feed_persist(FeedIdentifier::YouTube(feed_identifier.clone()), guild_id, channel_id).await?;
    guard.push_queue(feed_identifier).await;
    Ok(result)
  }

  pub async fn register_feed_twitter(&self, feed_identifier: FeedIdentifierTwitter, guild_id: GuildId, channel_id: ChannelId) -> MelodyResult<RegisterFeedResult> {
    let Some(guard) = self.write_twitter().await else { return Ok(RegisterFeedResult::FeedNotEnabled) };
    let result = self.register_feed_persist(FeedIdentifier::Twitter(feed_identifier.clone()), guild_id, channel_id).await?;
    guard.push_queue(feed_identifier).await;
    Ok(result)
  }

  async fn register_feed_persist(&self, feed_identifier: FeedIdentifier, guild_id: GuildId, channel_id: ChannelId) -> MelodyResult<RegisterFeedResult> {
    self.context.core.operate_persist_commit(async |persist| {
      Ok(persist.register_feed(feed_identifier, guild_id, channel_id))
    }).await
  }

  pub async fn unregister_guild_feeds(&self, guild_id: GuildId) -> MelodyResult<usize> {
    let mut feed_identifiers_youtube = AHashSet::new();
    let mut feed_identifiers_twitter = AHashSet::new();

    self.context.core.operate_persist_commit(async |persist| {
      feed_identifiers_youtube.extend(persist.feed_states.remove_guild_youtube_feeds(guild_id));
      feed_identifiers_twitter.extend(persist.feed_states.remove_guild_twitter_feeds(guild_id));

      Ok(())
    }).await?;

    let feed_count = feed_identifiers_youtube.len() + feed_identifiers_twitter.len();

    tokio::join!(
      async {
        if let Some(guard_youtube) = self.write_youtube().await {
          guard_youtube.retain_queue(|feed_identifier| {
            !feed_identifiers_youtube.contains(feed_identifier)
          }).await;
        };
      },
      async {
        if let Some(guard_twitter) = self.write_twitter().await {
          guard_twitter.retain_queue(|feed_identifier| {
            !feed_identifiers_twitter.contains(feed_identifier)
          }).await;
        };
      }
    );

    Ok(feed_count)
  }

  pub async fn unregister_feed(&self, feed_identifier: &FeedIdentifier, guild_id: GuildId) -> MelodyResult<UnregisterFeedResult> {
    match feed_identifier {
      FeedIdentifier::YouTube(feed_identifier) => self.unregister_feed_youtube(feed_identifier, guild_id).await,
      FeedIdentifier::Twitter(feed_identifier) => self.unregister_feed_twitter(feed_identifier, guild_id).await
    }
  }

  pub async fn unregister_feed_youtube(&self, feed_identifier: &FeedIdentifierYouTube, guild_id: GuildId) -> MelodyResult<UnregisterFeedResult> {
    let result = self.unregister_feed_persist(&FeedIdentifier::YouTube(feed_identifier.clone()), guild_id).await?;
    if let Some(guard) = self.write_youtube().await {
      guard.remove_queue(feed_identifier).await;
    };

    Ok(result)
  }

  pub async fn unregister_feed_twitter(&self, feed_identifier: &FeedIdentifierTwitter, guild_id: GuildId) -> MelodyResult<UnregisterFeedResult> {
    let result = self.unregister_feed_persist(&FeedIdentifier::Twitter(feed_identifier.clone()), guild_id).await?;
    if let Some(guard) = self.write_twitter().await {
      guard.remove_queue(feed_identifier).await;
    };

    Ok(result)
  }

  async fn unregister_feed_persist(&self, feed_identifier: &FeedIdentifier, guild_id: GuildId) -> MelodyResult<UnregisterFeedResult> {
    self.context.core.operate_persist_commit(async |persist| {
      Ok(persist.unregister_feed(feed_identifier, guild_id))
    }).await
  }

  pub async fn get_guild_feeds(&self, guild_id: GuildId) -> Vec<(FeedIdentifier, ChannelId, DateTime<Utc>)> {
    self.context.core.operate_persist(async |persist| {
      persist.feed_states.iter()
        .filter_map(|(feed_identifier, feed_state)| {
          feed_state.guilds.get(&guild_id).map(|&channel_id| {
            (feed_identifier, channel_id, feed_state.last_update)
          })
        })
        .collect()
    }).await
  }

  async fn write_youtube(&self) -> Option<RwLockMappedWriteGuard<'_, HandleYouTube>> {
    RwLockWriteGuard::try_map(self.handle_youtube.write().await, Option::as_mut).ok()
  }

  async fn write_twitter(&self) -> Option<RwLockMappedWriteGuard<'_, HandleTwitter>> {
    RwLockWriteGuard::try_map(self.handle_twitter.write().await, Option::as_mut).ok()
  }
}



#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegisterFeedResult {
  FeedChannelRegistered,
  FeedChannelReplaced(ChannelId),
  FeedNotEnabled
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnregisterFeedResult {
  FeedUnregistered(ChannelId),
  FeedChannelUnregistered(ChannelId),
  FeedNotRegistered,
  FeedChannelNotRegistered
}



type HandleYouTube = HandleWithContext<FeedModelYouTube, FeedContext>;
type HandleTwitter = HandleWithContext<FeedModelTwitter, FeedContext>;

#[derive(Debug, Clone)]
struct FeedContext {
  core: Core,
  client: Client
}

#[serenity::async_trait]
impl Context<FeedModelYouTube> for FeedContext {
  type Error = MelodyError;

  fn client(&self) -> &Client {
    &self.client
  }

  fn on_manager_error(&self, error: Self::Error) {
    error!("{error}");
  }

  async fn on_new_entries(&self, feed_identifier: &FeedIdentifierYouTube, entries: Vec<YouTubeVideo>) {
    let (message_cooldown, config) = self.core.operate_config(async |config| {
      (config.rss.message_cooldown, config.rss.youtube.clone())
    }).await;

    let Some(config) = config else { return };

    let guilds = self.core.operate_persist(async |persist| {
      persist.feed_states.youtube.get(feed_identifier)
        .map_or_else(AHashMap::new, |feed_state| feed_state.guilds.clone())
    }).await;

    for entry in entries {
      let mut link = entry.link;
      link.set_host(Some(&config.display_domain)).log_warn();

      for channel_id in guilds.values().copied() {
        channel_id.say(&self.core, link.as_str()).await
          .context("failed to send youtube video message")
          .log_error();

        sleep(message_cooldown).await;
      };
    };
  }

  async fn save_update_datetime(&self, feed_identifier: &FeedIdentifierYouTube, update: DateTime<Utc>) -> MelodyResult {
    self.core.operate_persist_commit(async |persist| {
      if let Some(feed_state) = persist.feed_states.youtube.get_mut(feed_identifier) {
        feed_state.last_update = update;
      };

      Ok(())
    }).await
  }

  async fn load_update_datetime(&self, feed_identifier: &FeedIdentifierYouTube) -> MelodyResult<Option<DateTime<Utc>>> {
    Ok(self.core.operate_persist(async |persist| {
      persist.feed_states.youtube.get(feed_identifier).map(|feed_state| feed_state.last_update)
    }).await)
  }
}

#[serenity::async_trait]
impl Context<FeedModelTwitter> for FeedContext {
  type Error = MelodyError;

  fn client(&self) -> &Client {
    &self.client
  }

  fn on_manager_error(&self, error: Self::Error) {
    error!("{error}");
  }

  async fn on_new_entries(&self, feed_identifier: &FeedIdentifierTwitter, entries: Vec<TwitterPost>) {
    let (message_cooldown, config) = self.core.operate_config(async |config| {
      (config.rss.message_cooldown, config.rss.twitter.clone())
    }).await;

    let Some(config) = config else { return };

    let guilds = self.core.operate_persist(async |persist| {
      persist.feed_states.twitter.get(feed_identifier)
        .map_or_else(AHashMap::new, |feed_state| feed_state.guilds.clone())
    }).await;

    for entry in entries {
      // when a tweet is a reqweet, the author will be that of the retweeted post
      if !entry.author.eq_ignore_ascii_case(&feed_identifier.handle) { continue };

      let mut link = entry.link;
      link.set_host(Some(&config.display_domain)).log_warn();

      for channel_id in guilds.values().copied() {
        channel_id.say(&self.core, link.as_str()).await
          .context("failed to send twitter post message")
          .log_error();

        sleep(message_cooldown).await;
      };
    };
  }

  async fn save_update_datetime(&self, feed_identifier: &FeedIdentifierTwitter, update: DateTime<Utc>) -> MelodyResult {
    self.core.operate_persist_commit(async |persist| {
      if let Some(feed_state) = persist.feed_states.twitter.get_mut(feed_identifier) {
        feed_state.last_update = update;
      };

      Ok(())
    }).await
  }

  async fn load_update_datetime(&self, feed_identifier: &FeedIdentifierTwitter) -> MelodyResult<Option<DateTime<Utc>>> {
    Ok(self.core.operate_persist(async |persist| {
      persist.feed_states.twitter.get(feed_identifier).map(|feed_state| feed_state.last_update)
    }).await)
  }
}

impl AsRef<Client> for FeedContext {
  fn as_ref(&self) -> &Client {
    &self.client
  }
}



macro_rules! match_feed_states {
  ($self:expr, $feed_identifier:expr, [$SelfPat:pat, $FeedIdentifier:pat] => $expr:expr) => (
    match ($self, $feed_identifier) {
      (FeedStates { youtube: $SelfPat, .. }, FeedIdentifier::YouTube($FeedIdentifier)) => $expr,
      (FeedStates { twitter: $SelfPat, .. }, FeedIdentifier::Twitter($FeedIdentifier)) => $expr,
    }
  );
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FeedStates {
  pub twitter: AHashMap<FeedIdentifierTwitter, FeedState>,
  pub youtube: AHashMap<FeedIdentifierYouTube, FeedState>
}

impl FeedStates {
  #[allow(unused)]
  pub fn get(&self, feed_identifier: &FeedIdentifier) -> Option<&FeedState> {
    match_feed_states!(self, feed_identifier, [feed, feed_identifier] => feed.get(feed_identifier))
  }

  pub fn get_mut(&mut self, feed_identifier: &FeedIdentifier) -> Option<&mut FeedState> {
    match_feed_states!(self, feed_identifier, [feed, feed_identifier] => feed.get_mut(feed_identifier))
  }

  pub fn get_or_insert_with(&mut self, feed_identifier: FeedIdentifier, f: impl FnOnce() -> FeedState) -> &mut FeedState {
    match_feed_states!(self, feed_identifier, [feed, feed_identifier] => feed.entry(feed_identifier).or_insert_with(f))
  }

  pub fn get_or_insert_default(&mut self, feed_identifier: FeedIdentifier) -> &mut FeedState {
    self.get_or_insert_with(feed_identifier, FeedState::default)
  }

  pub fn insert(&mut self, feed_identifier: FeedIdentifier, feed_state: FeedState) -> Option<FeedState> {
    match_feed_states!(self, feed_identifier, [feed, feed_identifier] => feed.insert(feed_identifier, feed_state))
  }

  pub fn remove(&mut self, feed_identifier: &FeedIdentifier) -> Option<FeedState> {
    match_feed_states!(self, feed_identifier, [feed, feed_identifier] => feed.remove(feed_identifier))
  }

  pub fn iter(&self) -> impl Iterator<Item = (FeedIdentifier, &FeedState)> {
    Iterator::chain(
      self.youtube.iter().map(|(feed_identifier, feed_state)| (FeedIdentifier::YouTube(feed_identifier.clone()), feed_state)),
      self.twitter.iter().map(|(feed_identifier, feed_state)| (FeedIdentifier::Twitter(feed_identifier.clone()), feed_state))
    )
  }

  pub fn remove_guild_youtube_feeds(&mut self, guild_id: GuildId) -> impl Iterator<Item = FeedIdentifierYouTube> {
    self.youtube
      .extract_if(move |_, feed_state| {
        feed_state.guilds.remove(&guild_id);
        feed_state.guilds.is_empty()
      })
      .map(|(feed_identifier, _)| feed_identifier)
  }

  pub fn remove_guild_twitter_feeds(&mut self, guild_id: GuildId) -> impl Iterator<Item = FeedIdentifierTwitter> {
    self.twitter
      .extract_if(move |_, feed_state| {
        feed_state.guilds.remove(&guild_id);
        feed_state.guilds.is_empty()
      })
      .map(|(feed_identifier, _)| feed_identifier)
  }
}

impl Extend<(FeedIdentifier, FeedState)> for FeedStates {
  fn extend<T: IntoIterator<Item = (FeedIdentifier, FeedState)>>(&mut self, iter: T) {
    for (feed_identifier, feed_state) in iter {
      self.insert(feed_identifier, feed_state);
    };
  }
}



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedState {
  pub last_update: DateTime<Utc>,
  pub guilds: AHashMap<GuildId, ChannelId>
}

impl FeedState {
  pub fn new(last_update: DateTime<Utc>) -> Self {
    FeedState { last_update, guilds: AHashMap::new() }
  }
}

impl Default for FeedState {
  fn default() -> Self {
    FeedState::new(Utc::now())
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum FeedIdentifier {
  YouTube(FeedIdentifierYouTube),
  Twitter(FeedIdentifierTwitter)
}

impl fmt::Display for FeedIdentifier {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Self::YouTube(feed) => fmt::Display::fmt(feed, f),
      Self::Twitter(feed) => fmt::Display::fmt(feed, f)
    }
  }
}



#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct FeedIdentifierYouTube {
  /// Channel ID, for example `UC7_YxT-KID8kRbqZo7MyscQ`.
  pub channel: String
}

impl fmt::Display for FeedIdentifierYouTube {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "www.youtube.com/channel/{}", self.channel)
  }
}

#[derive(Debug)]
struct FeedModelYouTube {
  config: ConfigRssYouTube
}

impl Model for FeedModelYouTube {
  type Identifier = FeedIdentifierYouTube;
  type Entry = YouTubeVideo;

  fn url(&self, identifier: &Self::Identifier) -> reqwest::Result<Url> {
    crate::utils::into_url(self.config.url(&identifier.channel))
  }

  fn delay(&self, queue_len: usize) -> Duration {
    self.config.delays.delay(queue_len)
  }
}



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedIdentifierTwitter {
  /// Twitter handle, for example `markiplier`.
  pub handle: String
}

impl fmt::Display for FeedIdentifierTwitter {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "twitter.com/{}", self.handle)
  }
}

impl Eq for FeedIdentifierTwitter {}

impl PartialEq for FeedIdentifierTwitter {
  fn eq(&self, other: &Self) -> bool {
    self.handle.eq_ignore_ascii_case(&other.handle)
  }
}

impl Hash for FeedIdentifierTwitter {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.handle.to_ascii_lowercase().hash(state);
  }
}

#[derive(Debug)]
struct FeedModelTwitter {
  config: ConfigRssTwitter
}

impl Model for FeedModelTwitter {
  type Identifier = FeedIdentifierTwitter;
  type Entry = TwitterPost;

  fn url(&self, identifier: &Self::Identifier) -> reqwest::Result<Url> {
    crate::utils::into_url(self.config.url(&identifier.handle))
  }

  fn delay(&self, queue_len: usize) -> Duration {
    self.config.delays.delay(queue_len)
  }
}
