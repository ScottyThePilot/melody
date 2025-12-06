//! Items associated with managing the bot's serenity state
mod activities;
mod config;
mod persist;

use crate::prelude::*;
use crate::feature::cleverbot::{CleverBotLoggerWrapper, CleverBotWrapper};
use crate::feature::feed::FeedManager;
use crate::feature::message_chains::{MessageChains, MessageChainsWrapper};
use crate::feature::music_player::MusicPlayer;
use crate::utils::youtube::YtDlp;
pub use self::activities::{ActivitiesContainer, Activities};
pub use self::config::*;
pub use self::persist::*;

use reqwest::Client as HttpClient;
use serenity::cache::Cache;
use serenity::client::{Client, Context};
use serenity::gateway::{ShardManager, ShardRunnerInfo};
use serenity::http::{CacheHttp, Http};
use serenity::model::guild::Member;
use serenity::model::id::{GuildId, UserId, ShardId};
use serenity::prelude::{TypeMap, TypeMapKey};
use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinHandle;

use std::fmt;
use std::sync::{Arc, OnceLock};



pub type ShardRunners = HashMap<ShardId, ShardRunnerInfo>;

macro_rules! key {
  ($vis:vis struct $Key:ident, $Value:ty) => {
    #[derive(Debug, Clone, Copy)]
    $vis struct $Key;

    impl TypeMapKey for $Key {
      type Value = $Value;
    }
  };
}

key!(pub struct MelodyFrameworkKey, crate::handler::MelodyFramework);
key!(pub struct ShardManagerKey, Arc<ShardManager>);

#[derive(Debug)]
pub struct State {
  pub previous_build_id: u64,
  pub config: ConfigContainer,
  pub persist: PersistContainer,
  pub persist_guilds: PersistGuildsWrapper,
  pub activities: ActivitiesContainer,
  pub cleverbot: CleverBotWrapper,
  pub cleverbot_logger: CleverBotLoggerWrapper,
  pub feed: OnceLock<FeedManager>,
  pub message_chains: MessageChainsWrapper,
  pub music_player: Option<Arc<MusicPlayer>>,
  pub yt_dlp: Option<YtDlp>,
  pub tasks: Mutex<Tasks>
}

impl State {
  pub async fn new(
    config: ConfigContainer,
    persist: PersistContainer,
    persist_guilds: PersistGuildsWrapper,
    activities: ActivitiesContainer,
    http_client: HttpClient
  ) -> MelodyResult<State> {
    let (cleverbot_delay, yt_dlp_path) = config.operate(async |config| {
      info!("YouTube RSS feeds are {}", if config.rss.youtube.is_some() { "enabled" } else { "disabled" });
      info!("Twitter RSS feeds are {}", if config.rss.twitter.is_some() { "enabled" } else { "disabled" });
      let yt_dlp_path = config.music_player.as_ref().map(|mp| mp.yt_dlp_path.clone());
      (config.cleverbot_ratelimit, yt_dlp_path)
    }).await;

    let previous_build_id = persist.operate_mut_commit(async |persist| Ok(persist.swap_build_id()))
      .await.context("failed to commit persist-guild state for build id")?;

    let cleverbot = CleverBotWrapper::new(cleverbot_delay);
    let cleverbot_logger = CleverBotLoggerWrapper::create()
      .await.context("failed to create cleverbot logger")?;

    let feed = OnceLock::new();

    let message_chains = MessageChains::new().into();

    let yt_dlp = yt_dlp_path.map(YtDlp::new);

    let music_player = yt_dlp.clone().map(|yt_dlp| {
      Arc::new(MusicPlayer::new(yt_dlp, http_client.clone()))
    });

    let tasks = Mutex::new(Tasks::default());

    Ok(State {
      previous_build_id,
      config,
      persist,
      persist_guilds,
      activities,
      cleverbot,
      cleverbot_logger,
      feed,
      message_chains,
      music_player,
      yt_dlp,
      tasks
    })
  }
}

macro_rules! for_each_some {
  ([$($value:expr),* $(,)?], $pat:pat => $expr:expr) => {
    $(if let Some($pat) = $value { $expr };)*
  };
}

#[derive(Debug, Default)]
pub struct Tasks {
  pub cycle_activities: Option<JoinHandle<()>>
}

impl Tasks {
  pub fn abort(&self) {
    for_each_some!([
      &self.cycle_activities
    ], task => task.abort());
  }
}

#[derive(Clone)]
pub struct CacheHttpData {
  pub data: Arc<RwLock<TypeMap>>,
  pub cache: Arc<Cache>,
  pub http: Arc<Http>
}

impl CacheHttp for CacheHttpData {
  fn cache(&self) -> Option<&Arc<Cache>> {
    Some(&self.cache)
  }

  fn http(&self) -> &Http {
    &self.http
  }
}

impl From<Context> for CacheHttpData {
  fn from(ctx: Context) -> Self {
    CacheHttpData {
      data: ctx.data,
      cache: ctx.cache,
      http: ctx.http
    }
  }
}

impl From<&Context> for CacheHttpData {
  fn from(ctx: &Context) -> Self {
    CacheHttpData {
      data: ctx.data.clone(),
      cache: ctx.cache.clone(),
      http: ctx.http.clone()
    }
  }
}

impl From<&Client> for CacheHttpData {
  fn from(client: &Client) -> Self {
    CacheHttpData {
      data: client.data.clone(),
      cache: client.cache.clone(),
      http: client.http.clone()
    }
  }
}

#[derive(Clone)]
pub struct Core {
  pub state: Arc<State>,
  pub data: Arc<RwLock<TypeMap>>,
  pub cache: Arc<Cache>,
  pub http: Arc<Http>
}

impl Core {
  pub fn new(cache_http_data: impl Into<CacheHttpData>, state: Arc<State>) -> Self {
    let CacheHttpData { data, cache, http } = cache_http_data.into();
    Core { state, data, cache, http }
  }

  #[inline]
  pub async fn get_checked<K>(&self) -> Option<K::Value>
  where K: TypeMapKey, K::Value: Clone {
    self.data.read().await.get::<K>().cloned()
  }

  /// Gets a value from the `TypeMap`, cloning it. Panics if it is not present.
  #[inline]
  pub async fn get<K>(&self) -> K::Value
  where K: TypeMapKey, K::Value: Clone {
    match self.get_checked::<K>().await {
      Some(value) => value,
      None => panic!("failed to get value from typemap with key {}", std::any::type_name::<K>())
    }
  }

  pub async fn get_shard_runners(&self) -> Arc<Mutex<ShardRunners>> {
    self.get::<ShardManagerKey>().await.runners.clone()
  }

  pub async fn trigger_shutdown(&self) {
    self.get::<ShardManagerKey>().await.shutdown_all().await
  }

  pub async fn randomize_activities(&self) {
    let shard_runners = self.get_shard_runners().await;
    let shard_runners_lock = shard_runners.lock().await;
    self.operate_activities(async |activities| {
      for (_, shard_runner) in shard_runners_lock.iter() {
        let activitiy = activities.select(self).log_error();
        shard_runner.runner_tx.set_activity(activitiy);
      };
    }).await;
  }

  pub async fn reload_activities(&self) -> MelodyResult {
    self.state.activities.refresh().await.context("failed to refresh activities state")?;
    Ok(())
  }

  pub async fn feed(&self) -> &FeedManager {
    self.operate_config(async |config| {
      self.state.feed.get_or_init(|| {
        FeedManager::new(self.clone(), HttpClient::new(), &config.rss)
      })
    }).await
  }

  pub async fn operate_config<R>(&self, operation: impl AsyncFnOnce(&Config) -> R) -> R {
    self.state.config.operate(operation).await
  }

  pub async fn operate_persist<R>(&self, operation: impl AsyncFnOnce(&Persist) -> R) -> R {
    self.state.persist.operate(operation).await
  }

  pub async fn operate_persist_commit<R>(&self, operation: impl AsyncFnOnce(&mut Persist) -> MelodyResult<R>) -> MelodyResult<R> {
    self.state.persist.operate_mut_commit(operation).await.context("failed to commit persist state")
  }

  pub async fn operate_persist_guild<R>(&self, id: GuildId, operation: impl AsyncFnOnce(&PersistGuild) -> MelodyResult<R>) -> MelodyResult<R> {
    PersistGuilds::get_default(&self.state.persist_guilds, id).await?.operate(operation).await
  }

  pub async fn operate_persist_guild_commit<R>(&self, id: GuildId, operation: impl AsyncFnOnce(&mut PersistGuild) -> MelodyResult<R>) -> MelodyResult<R> {
    PersistGuilds::get_default(&self.state.persist_guilds, id).await?
      .operate_mut_commit(operation).await.context("failed to commit persist-guild state")
  }

  pub async fn operate_activities<R>(&self, operation: impl AsyncFnOnce(&Activities) -> R) -> R {
    self.state.activities.operate(operation).await
  }

  pub async fn operate_tasks<F, R>(&self, operation: F) -> R
  where F: FnOnce(&mut Tasks) -> R {
    let mut guard = self.state.tasks.lock().await;
    operation(&mut *guard)
  }

  pub fn is_new_build(&self) -> bool {
    self.state.previous_build_id != crate::BUILD_ID
  }

  pub async fn current_member(&self, guild_id: GuildId) -> MelodyResult<Member> {
    guild_id.member(self, self.current_user_id())
      .await.context(format!("failed to locate member for current user in guild ({guild_id})"))
  }

  pub fn current_user_id(&self) -> UserId {
    self.cache.current_user().id
  }

  /// Aborts all tasks that this core might be responsible for
  pub async fn abort(&self) {
    self.state.tasks.lock().await.abort();
  }
}

impl fmt::Debug for Core {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.debug_struct("Core")
      .field("state", &self.state)
      .field("data", &format_args!(".."))
      .field("cache", &self.cache)
      .field("http", &self.http)
      .finish()
  }
}

impl AsRef<Cache> for Core {
  fn as_ref(&self) -> &Cache {
    &self.cache
  }
}

impl AsRef<Http> for Core {
  fn as_ref(&self) -> &Http {
    &self.http
  }
}

impl CacheHttp for Core {
  fn cache(&self) -> Option<&Arc<Cache>> {
    Some(&self.cache)
  }

  fn http(&self) -> &Http {
    &self.http
  }
}

impl From<&Core> for Core {
  fn from(core: &Core) -> Self {
    core.clone()
  }
}

impl<'a> From<crate::handler::MelodyContext<'a>> for Core {
  fn from(value: crate::handler::MelodyContext) -> Self {
    Core::new(value.serenity_context(), Arc::clone(value.data()))
  }
}

impl<'a> From<crate::handler::MelodyHandlerContext<'a>> for Core {
  fn from(value: crate::handler::MelodyHandlerContext<'a>) -> Self {
    Core::new(value.context, value.state)
  }
}

impl<'a> From<&crate::handler::MelodyHandlerContext<'a>> for Core {
  fn from(value: &crate::handler::MelodyHandlerContext<'a>) -> Self {
    Core::new(value.context.clone(), value.state.clone())
  }
}
