//! Items associated with managing the bot's serenity state
mod config;
mod persist;

use crate::MelodyResult;
use crate::feature::message_chains::MessageChains;
use crate::utils::Contextualize;
pub use self::config::*;
pub use self::persist::*;

use serenity::client::{Client, Context};
use serenity::client::bridge::gateway::ShardManager;
use serenity::model::id::GuildId;
use serenity::cache::Cache;
use serenity::http::Http;
use serenity::prelude::{TypeMap, TypeMapKey};
use serde::ser::Serialize;
use serde::de::DeserializeOwned;
use singlefile::manager::FileFormat;
use tokio::sync::{Mutex, RwLock};
use xz2::write::XzEncoder;
use xz2::read::XzDecoder;

use std::fmt;
use std::io::{Read, Write};
use std::sync::Arc;



#[derive(Debug, Error)]
pub enum TomlError {
  #[error(transparent)]
  Io(#[from] std::io::Error),
  #[error(transparent)]
  De(#[from] toml::de::Error),
  #[error(transparent)]
  Ser(#[from] toml::ser::Error)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Toml;

impl<T> FileFormat<T> for Toml
where T: Serialize + DeserializeOwned {
  type FormatError = TomlError;

  fn from_reader<R: Read>(&self, mut reader: R) -> Result<T, Self::FormatError> {
    let mut buf = String::new();
    reader.read_to_string(&mut buf)?;
    toml::from_str(&buf).map_err(From::from)
  }

  fn to_writer<W: Write>(&self, mut writer: W, value: &T) -> Result<(), Self::FormatError> {
    let buf = toml::to_string_pretty(value)?;
    writer.write_all(buf.as_bytes())?;
    Ok(())
  }

  fn to_buffer(&self, value: &T) -> Result<Vec<u8>, Self::FormatError> {
    match toml::to_string_pretty(value) {
      Ok(buf) => Ok(buf.into_bytes()),
      Err(error) => Err(error.into())
    }
  }
}



pub type CborError = serde_cbor::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cbor;

impl<T> FileFormat<T> for Cbor
where T: Serialize + DeserializeOwned {
  type FormatError = serde_cbor::Error;

  fn from_reader<R: Read>(&self, reader: R) -> Result<T, Self::FormatError> {
    serde_cbor::from_reader(reader).map_err(From::from)
  }

  fn to_writer<W: Write>(&self, writer: W, value: &T) -> Result<(), Self::FormatError> {
    serde_cbor::to_writer(writer, value).map_err(From::from)
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CborXz;

impl<T> FileFormat<T> for CborXz
where T: Serialize + DeserializeOwned {
  type FormatError = serde_cbor::Error;

  fn from_reader<R: Read>(&self, reader: R) -> Result<T, Self::FormatError> {
    serde_cbor::from_reader(XzDecoder::new(reader)).map_err(From::from)
  }

  fn to_writer<W: Write>(&self, writer: W, value: &T) -> Result<(), Self::FormatError> {
    serde_cbor::to_writer(XzEncoder::new(writer, 9), value).map_err(From::from)
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

key!(pub struct ConfigKey, ConfigContainer);
key!(pub struct PersistKey, PersistContainer);
key!(pub struct PersistGuildsKey, PersistGuildsWrapper);
key!(pub struct ShardManagerKey, Arc<Mutex<ShardManager>>);
key!(pub struct MessageChainsKey, crate::feature::message_chains::MessageChainsWrapper);
key!(pub struct PreviousBuildIdKey, u64);
key!(pub struct RestartKey, bool);



#[derive(Clone)]
pub struct Core {
  pub data: Arc<RwLock<TypeMap>>,
  pub cache: Arc<Cache>,
  pub http: Arc<Http>
}

impl Core {
  pub async fn insert_all(&self, core_data: CoreData) {
    let previous_build_id = core_data.persist.operate_mut(|persist| {
      persist.swap_build_id()
    }).await;

    let mut type_map = self.data.write().await;
    type_map.insert::<ConfigKey>(core_data.config);
    type_map.insert::<PersistKey>(core_data.persist);
    type_map.insert::<PersistGuildsKey>(core_data.persist_guilds.into());
    type_map.insert::<MessageChainsKey>(MessageChains::new().into());
    type_map.insert::<ShardManagerKey>(core_data.shard_manager);
    type_map.insert::<PreviousBuildIdKey>(previous_build_id);
    type_map.insert::<RestartKey>(false);
  }

  #[inline]
  pub async fn get_checked<K>(&self) -> Option<K::Value>
  where K: TypeMapKey, K::Value: Clone {
    self.data.read().await.get::<K>().cloned()
  }

  #[inline]
  pub async fn get<K>(&self) -> K::Value
  where K: TypeMapKey, K::Value: Clone {
    self.get_checked::<K>().await.expect("failed to get value from typemap")
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

  pub async fn trigger_shutdown(&self) {
    self.get::<ShardManagerKey>().await.lock().await.shutdown_all().await
  }

  pub async fn trigger_shutdown_restart(&self) {
    self.trigger_shutdown().await;
    self.data.write().await.insert::<RestartKey>(true)
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
      cache: client.cache_and_http.cache.clone(),
      http: client.cache_and_http.http.clone()
    }
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

#[derive(Debug)]
pub struct CoreData {
  pub config: ConfigContainer,
  pub persist: PersistContainer,
  pub persist_guilds: PersistGuilds,
  pub shard_manager: Arc<Mutex<ShardManager>>
}

pub async fn operate_lock<T, F, R>(container: Arc<Mutex<T>>, operation: F) -> R
where F: FnOnce(&mut T) -> R {
  let mut guard = container.lock().await;
  operation(&mut *guard)
}

pub async fn operate_read<T, F, R>(container: Arc<RwLock<T>>, operation: F) -> R
where F: FnOnce(&T) -> R {
  let guard = container.read().await;
  operation(&*guard)
}

pub async fn operate_write<T, F, R>(container: Arc<RwLock<T>>, operation: F) -> R
where F: FnOnce(&mut T) -> R {
  let mut guard = container.write().await;
  operation(&mut *guard)
}

pub async fn data_take<K: TypeMapKey>(data: &Arc<RwLock<TypeMap>>) -> K::Value {
  data.write().await.remove::<K>().expect("failed to take value from typemap")
}
