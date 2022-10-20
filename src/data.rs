mod config;
mod persist;

use crate::MelodyResult;
use crate::utils::Contextualize;
pub use self::config::*;
pub use self::persist::*;

use serenity::client::{Client, Context};
use serenity::client::bridge::gateway::ShardManager;
use serenity::model::id::GuildId;
use serenity::prelude::TypeMapKey;
use serde::ser::Serialize;
use serde::de::DeserializeOwned;
use singlefile::manager::FileFormat;
use tokio::sync::{Mutex, RwLock};
use xz2::write::XzEncoder;
use xz2::read::XzDecoder;

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

  fn to_buf(&self, value: &T) -> Result<Vec<u8>, Self::FormatError> {
    match toml::to_string_pretty(value) {
      Ok(buf) => Ok(buf.into_bytes()),
      Err(error) => Err(error.into())
    }
  }

  fn to_writer<W: Write>(&self, mut writer: W, value: &T) -> Result<(), Self::FormatError> {
    let buf = toml::to_string_pretty(value)?;
    writer.write_all(buf.as_bytes())?;
    Ok(())
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



macro_rules! data_get_fn {
  ($vis:vis async fn $function:ident, $Key:ty, $Value:ty) => {
    #[inline]
    $vis async fn $function(ctx: &Context) -> $Value {
      data_get::<$Key>(ctx).await
    }
  };
}

data_get_fn!(pub async fn data_get_config, ConfigKey, ConfigContainer);
data_get_fn!(pub async fn data_get_persist, PersistKey, PersistContainer);

macro_rules! context {
  ($function:ident) => (concat!("failed to save state in ", stringify!($function)));
}

macro_rules! data_operate_fn {
  ($vis:vis async fn $function:ident($($arg:ident: $Arg:ty),*), $getter:ident, $method:ident, $Type:ty) => {
    $vis async fn $function<F, R>(ctx: &Context, $($arg: $Arg,)* operation: F) -> R
    where F: FnOnce($Type) -> R {
      $getter(ctx $(, $arg)*).await.$method(operation).await
    }
  };
  ($vis:vis async fn $function:ident($($arg:ident: $Arg:ty),*), $getter:ident, $method:ident?, $Type:ty, $err:expr) => {
    $vis async fn $function<F, R>(ctx: &Context, $($arg: $Arg,)* operation: F) -> MelodyResult<R>
    where F: FnOnce($Type) -> MelodyResult<R> {
      $getter(ctx $(, $arg)*).await.$method(operation).await.context(context!($function))
    }
  };
  ($vis:vis async fn $function:ident($($arg:ident: $Arg:ty),*), $getter:ident?, $method:ident, $Type:ty) => {
    $vis async fn $function<F, R>(ctx: &Context, $($arg: $Arg,)* operation: F) -> MelodyResult<R>
    where F: FnOnce($Type) -> MelodyResult<R> {
      $getter(ctx $(, $arg)*).await?.$method(operation).await
    }
  };
  ($vis:vis async fn $function:ident($($arg:ident: $Arg:ty),*), $getter:ident?, $method:ident?, $Type:ty, $err:expr) => {
    $vis async fn $function<F, R>(ctx: &Context, $($arg: $Arg,)* operation: F) -> MelodyResult<R>
    where F: FnOnce($Type) -> MelodyResult<R> {
      $getter(ctx $(, $arg)*).await?.$method(operation).await.context(context!($function))
    }
  };
}

data_operate_fn!(pub async fn data_operate_config(), data_get_config, operate, &Config);
data_operate_fn!(pub async fn data_operate_persist(), data_get_persist, operate, &Persist);
data_operate_fn!(pub async fn data_operate_persist_mut(), data_get_persist, operate_mut, &mut Persist);
data_operate_fn!(pub async fn data_operate_persist_guild(id: GuildId), data_get_persist_guild?, operate, &PersistGuild);
data_operate_fn!(pub async fn data_operate_persist_guild_mut(id: GuildId), data_get_persist_guild?, operate_mut, &mut PersistGuild);

data_operate_fn!(
  pub async fn data_operate_persist_commit(),
  data_get_persist, operate_mut_commit?, &mut Persist,
  "failed to commit persist state"
);

data_operate_fn!(
  pub async fn data_operate_persist_guild_commit(id: GuildId),
  data_get_persist_guild?, operate_mut_commit?, &mut PersistGuild,
  "failed to commit persist-guild state"
);

pub async fn data_get_persist_guild(ctx: &Context, id: GuildId) -> MelodyResult<PersistGuildContainer> {
  let persist_guilds = data_get::<PersistGuildsKey>(ctx).await;
  PersistGuilds::get_default(persist_guilds, id).await
}

pub async fn trigger_shutdown(ctx: &Context) {
  data_get::<ShardManagerKey>(ctx).await.lock().await.shutdown_all().await
}
