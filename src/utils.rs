pub mod youtube;

use crate::prelude::*;

use chrono::{DateTime, Utc};
use itertools::Itertools;
use once_cell::sync::Lazy;
use rand::SeedableRng;
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use regex::Regex;
use serenity::model::id::{EmojiId, GuildId, UserId};
use serenity::cache::Cache;

use std::fmt;



pub fn create_rng() -> SmallRng {
  SmallRng::from_rng(rand::thread_rng()).expect("failed to seed smallrng")
}

pub fn shuffle<T>(list: &mut [T]) {
  list.shuffle(&mut rand::thread_rng());
}

pub fn capitalize(s: impl AsRef<str>) -> String {
  let mut chars = s.as_ref().chars();
  chars.next().map_or_else(String::new, |first| {
    first.to_uppercase()
      .chain(chars.map(|c| c.to_ascii_lowercase()))
      .collect()
  })
}

pub fn kebab_case_to_words(s: impl AsRef<str>) -> String {
  s.as_ref().split("-").map(capitalize).join(" ")
}

pub fn guild_name(cache: impl AsRef<Cache>, guild_id: GuildId) -> String {
  cache.as_ref().guild(guild_id).map_or_else(|| "Unknown".to_owned(), |guild| guild.name.clone())
}

pub fn parse_emojis(message: &str) -> Vec<EmojiId> {
  static RX: Lazy<Regex> = Lazy::new(|| Regex::new(r"<a?:[0-9a-zA-Z_]+:(\d+)>").unwrap());
  RX.captures_iter(message)
    .filter_map(|captures| captures.get(1).map(<&str>::from))
    .filter_map(|id| id.parse::<u64>().ok().map(EmojiId::new))
    .collect::<Vec<EmojiId>>()
}

/// Takes message content and a user ID, returning the remainder of the message if the string
/// mentioned the user at the start of it.
pub fn strip_user_mention(msg: &str, user_id: UserId) -> Option<&str> {
  let msg = msg.trim().strip_prefix("<@")?;
  let msg = msg.strip_prefix("!").unwrap_or(msg);
  let msg = msg.strip_prefix(&user_id.to_string())?;
  let msg = msg.strip_prefix(">")?.trim_start();
  Some(msg)
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoDebug<T>(pub T);

impl<T> std::ops::Deref for NoDebug<T> {
  type Target = T;

  #[inline]
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<T> std::ops::DerefMut for NoDebug<T> {
  #[inline]
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl<T: fmt::Display> fmt::Display for NoDebug<T> {
  #[inline]
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    <T as fmt::Display>::fmt(&self.0, f)
  }
}

impl<T> fmt::Debug for NoDebug<T> {
  #[inline]
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.write_str("..")
  }
}

#[derive(Debug, Clone, Copy)]
pub struct Blockify<S>(pub S);

impl<S: fmt::Display> Blockify<S> {
  pub const fn new(s: S) -> Self {
    Blockify(s)
  }
}

impl<S: fmt::Display> fmt::Display for Blockify<S> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "`{}`", self.0)
  }
}

#[derive(Debug, Clone, Copy)]
pub struct Timestamp(pub DateTime<Utc>, pub TimestampFormat);

impl Timestamp {
  pub const fn new(timestamp: DateTime<Utc>, format: TimestampFormat) -> Self {
    Timestamp(timestamp, format)
  }
}

impl fmt::Display for Timestamp {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<t:{}:{}>", self.0.timestamp(), self.1)
  }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum TimestampFormat {
  /// `8:17 PM`
  ShortTime,
  /// `8:17:00 PM`
  LongTime,
  /// `7/20/69`
  ShortDate,
  /// `July 20, 1969`
  LongDate,
  /// `July 20, 1969 at 8:17 PM`
  ShortDateTime,
  /// `Sunday, July 20, 1969 at 8:17 PM`
  LongDateTime,
  /// `55 years ago`
  Relative
}

impl fmt::Display for TimestampFormat {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.write_str(match self {
      TimestampFormat::ShortTime => "t",
      TimestampFormat::LongTime => "T",
      TimestampFormat::ShortDate => "d",
      TimestampFormat::LongDate => "D",
      TimestampFormat::ShortDateTime => "f",
      TimestampFormat::LongDateTime => "F",
      TimestampFormat::Relative => "R"
    })
  }
}

pub trait Contextualize {
  type Output;

  fn context(self, context: impl Into<String>) -> Self::Output;
}

impl<T> Contextualize for std::io::Result<T> {
  type Output = MelodyResult<T>;

  fn context(self, context: impl Into<String>) -> Self::Output {
    self.map_err(|error| {
      MelodyError::FileError(MelodyFileError::Io(error), context.into())
    })
  }
}

impl<T, FE> Contextualize for Result<T, singlefile::Error<FE>>
where singlefile::Error<FE>: Into<MelodyFileError> {
  type Output = MelodyResult<T>;

  fn context(self, context: impl Into<String>) -> Self::Output {
    self.map_err(|error| {
      MelodyError::FileError(error.into(), context.into())
    })
  }
}

impl<T, FE> Contextualize for Result<T, singlefile::UserError<FE, MelodyError>>
where singlefile::Error<FE>: Into<MelodyFileError> {
  type Output = MelodyResult<T>;

  fn context(self, context: impl Into<String>) -> Self::Output {
    self.map_err(|error| {
      MelodyError::FileError(match error {
        singlefile::UserError::Format(error) => singlefile::Error::Format(error).into(),
        singlefile::UserError::Io(error) => singlefile::Error::Io(error).into(),
        singlefile::UserError::User(error) => return error
      }, context.into())
    })
  }
}

impl<T> Contextualize for Result<T, cleverbot_logs::Error> {
  type Output = MelodyResult<T>;

  fn context(self, context: impl Into<String>) -> Self::Output {
    self.map_err(|error| {
      MelodyError::FileError(MelodyFileError::CleverBotLog(error), context.into())
    })
  }
}

impl<T> Contextualize for Result<T, serenity::Error> {
  type Output = MelodyResult<T>;

  fn context(self, context: impl Into<String>) -> Self::Output {
    self.map_err(|error| MelodyError::SerenityError(error, context.into()))
  }
}

#[macro_export]
macro_rules! operate {
  ($core:expr, $function:ident::<$Key:ty>, $operation:expr) => {
    crate::data::$function($core.get::<$Key>().await, $operation).await
  };
}
