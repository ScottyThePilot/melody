//! Items associated with managing the bot's serenity state
mod activities;
mod config;
mod persist;

use crate::MelodyResult;
use crate::ratelimiter::RateLimiter;
use crate::utils::Contextualize;
pub use self::activities::{ActivitiesContainer, Activities};
pub use self::config::*;
pub use self::persist::*;

use serenity::client::{Client, Context};
use serenity::gateway::{ShardManager, ShardRunnerInfo};
use serenity::model::id::{GuildId, UserId, ShardId};
use serenity::model::guild::Member;
use serenity::cache::Cache;
use serenity::http::{CacheHttp, Http};
use serenity::prelude::{TypeMap, TypeMapKey};
use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinHandle;

use std::fmt;
use std::collections::HashMap;
use std::sync::Arc;



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

key!(pub struct ConfigKey, ConfigContainer);
key!(pub struct PersistKey, PersistContainer);
key!(pub struct PersistGuildsKey, PersistGuildsWrapper);
key!(pub struct ActivitiesKey, ActivitiesContainer);
key!(pub struct ShardManagerKey, Arc<ShardManager>);
key!(pub struct CleverBotKey, crate::feature::cleverbot::CleverBotWrapper);
key!(pub struct CleverBotLoggerKey, crate::feature::cleverbot::CleverBotLoggerWrapper);
key!(pub struct FeedKey, crate::feature::feed::FeedWrapper);
key!(pub struct MessageChainsKey, crate::feature::message_chains::MessageChainsWrapper);
key!(pub struct MusicPlayerKey, Option<Arc<crate::feature::music_player::MusicPlayer>>);
key!(pub struct TasksKey, TasksWrapper);
key!(pub struct PreviousBuildIdKey, u64);



macro_rules! for_each_some {
  ([$($value:expr),* $(,)?], $pat:pat => $expr:expr) => {
    $(if let Some($pat) = $value { $expr };)*
  };
}

pub type TasksWrapper = Arc<Mutex<Tasks>>;

#[derive(Debug, Default)]
pub struct Tasks {
  pub cycle_activities: Option<JoinHandle<()>>,
  pub respawn_feed_tasks: Option<JoinHandle<()>>
}

impl Tasks {
  pub fn abort(&self) {
    for_each_some!([
      &self.cycle_activities,
      &self.respawn_feed_tasks
    ], task => task.abort());
  }
}

#[derive(Clone)]
pub struct Core {
  pub data: Arc<RwLock<TypeMap>>,
  pub cache: Arc<Cache>,
  pub http: Arc<Http>
}

impl Core {
  /// Gives the caller mutable access to the `TypeMap` so that they can initialize it.
  pub async fn init<F>(&self, operation: F)
  where F: FnOnce(&mut TypeMap) {
    let mut type_map = self.data.write().await;
    operation(&mut *type_map)
  }

  #[inline]
  pub async fn take_checked<K>(&self) -> Option<K::Value> where K: TypeMapKey {
    self.data.write().await.remove::<K>()
  }

  /// Takes a value from the `TypeMap`, cloning it. Panics if it is not present.
  #[inline]
  pub async fn take<K>(&self) -> K::Value where K: TypeMapKey {
    self.take_checked::<K>().await.expect("failed to take value from typemap")
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

  #[inline]
  pub async fn get_default<K>(&self) -> K::Value
  where K: TypeMapKey, K::Value: Default + Clone {
    self.data.write().await.entry::<K>().or_default().clone()
  }

  pub async fn get_shard_runners(&self) -> Arc<Mutex<ShardRunners>> {
    self.get::<ShardManagerKey>().await.runners.clone()
  }

  pub async fn get_config(&self) -> ConfigContainer {
    self.get::<ConfigKey>().await
  }

  pub async fn get_persist(&self) -> PersistContainer {
    self.get::<PersistKey>().await
  }

  pub async fn get_persist_guild(&self, id: GuildId) -> MelodyResult<PersistGuildContainer> {
    let persist_guilds = self.get::<PersistGuildsKey>().await;
    PersistGuilds::get_default(persist_guilds, id).await
  }

  pub async fn get_activities(&self) -> ActivitiesContainer {
    self.get::<ActivitiesKey>().await
  }

  pub async fn trigger_shutdown(&self) {
    self.get::<ShardManagerKey>().await.shutdown_all().await
  }

  pub async fn randomize_activities(&self) {
    let shard_runners = self.get_shard_runners().await;
    let shard_runners_lock = shard_runners.lock().await;
    self.operate_activities(|activities| {
      for (_, shard_runner) in shard_runners_lock.iter() {
        let activitiy = log_result!(activities.select(self));
        shard_runner.runner_tx.set_activity(activitiy);
      };
    }).await;
  }

  pub async fn is_new_build(&self) -> bool {
    self.get::<PreviousBuildIdKey>().await != crate::BUILD_ID
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
    if let Some(tasks) = self.get_checked::<TasksKey>().await {
      tasks.lock().await.abort();
    };
    if let Some(feed_wrapper) = self.get_checked::<FeedKey>().await {
      feed_wrapper.lock().await.abort_all();
    };
  }
}

impl fmt::Debug for Core {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.debug_struct("Core")
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

impl From<Context> for Core {
  fn from(ctx: Context) -> Self {
    Core {
      data: ctx.data,
      cache: ctx.cache,
      http: ctx.http
    }
  }
}

impl From<&Context> for Core {
  fn from(ctx: &Context) -> Self {
    Core {
      data: ctx.data.clone(),
      cache: ctx.cache.clone(),
      http: ctx.http.clone()
    }
  }
}

impl From<&Client> for Core {
  fn from(client: &Client) -> Self {
    Core {
      data: client.data.clone(),
      cache: client.cache.clone(),
      http: client.http.clone()
    }
  }
}

impl From<&Core> for Core {
  fn from(core: &Core) -> Self {
    core.clone()
  }
}



macro_rules! member_operate_fn {
  ($vis:vis async fn $function:ident($($arg:ident: $Arg:ty),*), $getter:ident, $method:ident, $Type:ty) => {
    #[allow(dead_code)]
    $vis async fn $function<F, R>(&self, $($arg: $Arg,)* operation: F) -> R
    where F: FnOnce($Type) -> R {
      self.$getter($($arg,)*).await.$method(operation).await
    }
  };
  ($vis:vis async fn $function:ident($($arg:ident: $Arg:ty),*), $getter:ident?, $method:ident, $Type:ty) => {
    #[allow(dead_code)]
    $vis async fn $function<F, R>(&self, $($arg: $Arg,)* operation: F) -> MelodyResult<R>
    where F: FnOnce($Type) -> MelodyResult<R> {
      self.$getter($($arg,)*).await?.$method(operation).await
    }
  };
}

impl Core {
  member_operate_fn!(pub async fn operate_config(), get_config, operate, &Config);
  member_operate_fn!(pub async fn operate_persist(), get_persist, operate, &Persist);
  member_operate_fn!(pub async fn operate_persist_mut(), get_persist, operate_mut, &mut Persist);
  member_operate_fn!(pub async fn operate_persist_guild(id: GuildId), get_persist_guild?, operate, &PersistGuild);
  member_operate_fn!(pub async fn operate_persist_guild_mut(id: GuildId), get_persist_guild?, operate_mut, &mut PersistGuild);
  member_operate_fn!(pub async fn operate_activities(), get_activities, operate, &Activities);

  pub async fn operate_persist_commit<F, R>(&self, operation: F) -> MelodyResult<R>
  where F: FnOnce(&mut Persist) -> MelodyResult<R> {
    self.get_persist().await
      .operate_mut_commit(operation).await
      .context("failed to commit persist state")
  }

  pub async fn operate_persist_guild_commit<F, R>(&self, id: GuildId, operation: F) -> MelodyResult<R>
  where F: FnOnce(&mut PersistGuild) -> MelodyResult<R> {
    self.get_persist_guild(id).await?
      .operate_mut_commit(operation).await
      .context("failed to commit persist-guild state")
  }
}

#[allow(dead_code)]
pub async fn operate_lock<T, F, R>(container: Arc<Mutex<T>>, operation: F) -> R
where F: FnOnce(&mut T) -> R {
  let mut guard = container.lock().await;
  operation(&mut *guard)
}

#[allow(dead_code)]
pub async fn operate_read<T, F, R>(container: Arc<RwLock<T>>, operation: F) -> R
where F: FnOnce(&T) -> R {
  let guard = container.read().await;
  operation(&*guard)
}

#[allow(dead_code)]
pub async fn operate_write<T, F, R>(container: Arc<RwLock<T>>, operation: F) -> R
where F: FnOnce(&mut T) -> R {
  let mut guard = container.write().await;
  operation(&mut *guard)
}

#[allow(dead_code)]
pub async fn operate_ratelimiter<T, F, R>(container: RateLimiter<T>, operation: F) -> R
where F: FnOnce(&mut T) -> R {
  let mut timeslice = container.get().await;
  operation(&mut *timeslice)
}
