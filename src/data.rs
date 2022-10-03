mod brain;
mod config;
mod persist;

use crate::MelodyResult;
use crate::utils::Contextualize;
pub use self::brain::*;
pub use self::config::*;
pub use self::persist::*;

use serenity::client::{Client, Context};
use serenity::client::bridge::gateway::ShardManager;
use serenity::model::id::GuildId;
use serenity::prelude::TypeMapKey;
use serde::ser::Serialize;
use serde::de::DeserializeOwned;
use singlefile::error::FormatError;
use singlefile::manager::FileFormat;
use tokio::sync::{Mutex, RwLockReadGuard, RwLockWriteGuard};
use xz2::write::XzEncoder;
use xz2::read::XzDecoder;

use std::io::{Read, Write};
use std::sync::Arc;



#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Toml;

impl FileFormat for Toml {
  fn from_reader<R, T>(&self, mut reader: R) -> Result<T, FormatError>
  where R: Read, T: DeserializeOwned {
    let mut buf = String::new();
    reader.read_to_string(&mut buf)?;
    toml::from_str(&buf).map_err(From::from)
  }

  fn to_writer<W, T>(&self, mut writer: W, value: &T) -> Result<(), FormatError>
  where W: Write, T: Serialize {
    let buf = toml::to_string_pretty(value)?;
    writer.write_all(buf.as_bytes())?;
    Ok(())
  }
}



#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bincode;

impl FileFormat for Bincode {
  fn from_reader<R, T>(&self, reader: R) -> Result<T, FormatError>
  where R: Read, T: DeserializeOwned {
    bincode::deserialize_from(reader).map_err(From::from)
  }

  fn to_writer<W, T>(&self, writer: W, value: &T) -> Result<(), FormatError>
  where W: Write, T: Serialize {
    bincode::serialize_into(writer, value).map_err(From::from)
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BincodeCompressed;

impl FileFormat for BincodeCompressed {
  fn from_reader<R, T>(&self, reader: R) -> Result<T, FormatError>
  where R: Read, T: DeserializeOwned {
    bincode::deserialize_from(XzDecoder::new(reader)).map_err(From::from)
  }

  fn to_writer<W, T>(&self, writer: W, value: &T) -> Result<(), FormatError>
  where W: Write, T: Serialize {
    bincode::serialize_into(XzEncoder::new(writer, 9), value).map_err(From::from)
  }
}



macro_rules! key {
  ($vis:vis struct $Key:ident, $Value:ty) => {
    #[derive(Debug, Clone, Copy)]
    $vis struct $Key;

    impl TypeMapKey for $Key {
      type Value = $Value;
    }
  };
}

key!(pub struct BrainKey, BrainContainer);
key!(pub struct ConfigKey, ConfigContainer);
key!(pub struct PersistKey, PersistContainer);
key!(pub struct PersistGuildsKey, PersistGuildsWrapper);
key!(pub struct ShardManagerKey, Arc<Mutex<ShardManager>>);
key!(pub struct PreviousBuildIdKey, u64);
key!(pub struct RestartKey, bool);

#[inline]
pub async fn data_insert<K>(client: &Client, value: K::Value)
where K: TypeMapKey {
  client.data.write().await.insert::<K>(value);
}

pub async fn data_take<K>(client: &Client) -> K::Value
where K: TypeMapKey {
  client.data.write().await.remove::<K>()
    .expect("failed to take value from typemap")
}

#[inline]
pub async fn data_get<K>(ctx: &Context) -> K::Value
where K: TypeMapKey, K::Value: Clone {
  ctx.data.read().await.get::<K>()
    .expect("failed to acquire value from typemap")
    .clone()
}

macro_rules! data_get_fn {
  ($vis:vis async fn $function:ident, $Key:ty, $Value:ty) => {
    #[inline]
    $vis async fn $function(ctx: &Context) -> $Value {
      data_get::<$Key>(ctx).await
    }
  };
}

data_get_fn!(pub async fn data_get_brain, BrainKey, BrainContainer);
data_get_fn!(pub async fn data_get_config, ConfigKey, ConfigContainer);
data_get_fn!(pub async fn data_get_persist, PersistKey, PersistContainer);

macro_rules! data_access_fn {
  ($vis:vis async fn $function:ident, $getter:ident, $access:ident, $Lock:ty) => {
    #[inline]
    $vis async fn $function<F, R>(ctx: &Context, f: F) -> R
    where F: FnOnce($Lock) -> R {
      f($getter(ctx).await.$access().await)
    }
  };
}

data_access_fn!(pub async fn data_access_brain, data_get_brain, read, RwLockReadGuard<Brain>);
data_access_fn!(pub async fn data_access_brain_mut, data_get_brain, write, RwLockWriteGuard<Brain>);
data_access_fn!(pub async fn data_access_config, data_get_config, access, RwLockReadGuard<Config>);
data_access_fn!(pub async fn data_access_config_mut, data_get_config, access_mut, RwLockWriteGuard<Config>);
data_access_fn!(pub async fn data_access_persist, data_get_persist, access, RwLockReadGuard<Persist>);
data_access_fn!(pub async fn data_access_persist_mut, data_get_persist, access_mut, RwLockWriteGuard<Persist>);

/// Acquires and provides access to the persist state's write lock,
/// committing its state to disk afterwards.
pub async fn data_modify_persist<F, R>(ctx: &Context, f: F) -> MelodyResult<R>
where F: FnOnce(RwLockWriteGuard<Persist>) -> MelodyResult<R> {
  let container = data_get_persist(ctx).await;
  let out = f(container.access_mut().await)?;
  trace!("Saving persist state...");
  container.commit().await.context("failed to commit persist state to disk")?;
  Ok(out)
}

#[inline]
pub async fn data_get_persist_guild(ctx: &Context, id: GuildId) -> Option<PersistGuildContainer> {
  let persist_guilds = data_get::<PersistGuildsKey>(ctx).await;
  PersistGuilds::get(persist_guilds, id).await
}

#[inline]
pub async fn data_try_get_persist_guild(ctx: &Context, id: GuildId) -> MelodyResult<PersistGuildContainer> {
  let persist_guilds = data_get::<PersistGuildsKey>(ctx).await;
  PersistGuilds::get_default(persist_guilds, id)
    .await.context("failed to retrieve persist-guild")
}

#[inline]
pub async fn data_access_persist_guild<F, R>(ctx: &Context, id: GuildId, f: F) -> MelodyResult<R>
where F: FnOnce(RwLockReadGuard<PersistGuild>) -> MelodyResult<R> {
  f(data_try_get_persist_guild(ctx, id).await?.access().await)
}

#[inline]
pub async fn data_access_persist_guild_mut<F, R>(ctx: &Context, id: GuildId, f: F) -> MelodyResult<R>
where F: FnOnce(RwLockWriteGuard<PersistGuild>) -> MelodyResult<R> {
  f(data_try_get_persist_guild(ctx, id).await?.access_mut().await)
}

/// Acquires and provides access to a persist-guild state's write lock,
/// committing its state to disk afterwards.
pub async fn data_modify_persist_guild<F, R>(ctx: &Context, id: GuildId, f: F) -> MelodyResult<R>
where F: FnOnce(RwLockWriteGuard<PersistGuild>) -> MelodyResult<R> {
  let container = data_try_get_persist_guild(ctx, id).await?;
  let out = f(container.access_mut().await)?;
  trace!("Saving persist-guild state for guild ({id})...");
  container.commit().await.context("failed to commit persist-guild state to disk")?;
  Ok(out)
}

pub async fn trigger_shutdown(ctx: &Context) {
  data_get::<ShardManagerKey>(ctx).await.lock().await.shutdown_all().await
}



pub mod serde_id {
  use serde::ser::{Serialize, Serializer};
  use serde::de::{Deserialize, Deserializer};

  pub fn serialize<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
  where T: Copy + Into<u64>, S: Serializer {
    u64::serialize(&value.clone().into(), serializer)
  }

  pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
  where T: From<u64>, D: Deserializer<'de> {
    u64::deserialize(deserializer).map(T::from)
  }
}
