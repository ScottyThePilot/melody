pub mod youtube;

use crate::prelude::*;

use chrono::{DateTime, Utc};
use defy::ContextualError;
use rand::seq::SliceRandom;
use regex::Regex;
use serenity::model::id::{EmojiId, RoleId, UserId};
use poise::slash_argument::{SlashArgument, SlashArgError};

use std::fmt;
use std::ops::Deref;
use std::sync::OnceLock;



#[derive(Debug)]
pub struct LazyRegex {
  lock: OnceLock<Regex>,
  pattern: &'static str
}

impl LazyRegex {
  #[inline]
  pub const fn new(pattern: &'static str) -> Self {
    LazyRegex { lock: OnceLock::new(), pattern }
  }

  #[inline]
  pub fn force(this: &Self) -> &Regex {
    this.lock.get_or_init(|| Regex::new(this.pattern).unwrap())
  }
}

impl Deref for LazyRegex {
  type Target = Regex;

  #[inline]
  fn deref(&self) -> &Self::Target {
    LazyRegex::force(self)
  }
}



#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RoleOrUser {
  User(UserId),
  Role(RoleId)
}

#[serenity::async_trait]
impl SlashArgument for RoleOrUser {
  async fn extract(
    ctx: &serenity::client::Context,
    interaction: &serenity::model::application::CommandInteraction,
    value: &serenity::model::application::ResolvedValue<'_>
  ) -> Result<Self, SlashArgError> {
    Result::or(
      <UserId as SlashArgument>::extract(ctx, interaction, value).await.map(RoleOrUser::User),
      <RoleId as SlashArgument>::extract(ctx, interaction, value).await.map(RoleOrUser::Role)
    )
  }

  fn create(builder: serenity::builder::CreateCommandOption) -> serenity::builder::CreateCommandOption {
    builder.kind(serenity::model::application::CommandOptionType::Mentionable)
  }
}

pub fn shuffle<T>(list: &mut [T]) {
  list.shuffle(&mut rand::rng());
}

pub fn parse_emojis(message: &str) -> Vec<EmojiId> {
  static RX: LazyRegex = LazyRegex::new(r"<a?:[0-9a-zA-Z_]+:(\d+)>");
  RX.captures_iter(message)
    .filter_map(|captures| captures.get(1).map(<&str>::from))
    .filter_map(|id| id.parse::<u64>().ok().map(EmojiId::new))
    .collect::<Vec<EmojiId>>()
}

#[derive(Debug, Clone, Copy)]
pub struct Blockify<S>(pub S);

impl<S: fmt::Display> fmt::Display for Blockify<S> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "`{}`", self.0)
  }
}

#[derive(Debug, Clone, Copy)]
pub struct Timestamp(pub DateTime<Utc>, pub TimestampFormat);

impl Timestamp {
  pub const fn new(datetime: DateTime<Utc>, format: TimestampFormat) -> Self {
    Timestamp(datetime, format)
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
      MelodyError::FileError(ContextualError::new(MelodyFileError::Io(error), context.into()))
    })
  }
}

impl<T, FE> Contextualize for Result<T, singlefile::Error<FE>>
where singlefile::Error<FE>: Into<MelodyFileError> {
  type Output = MelodyResult<T>;

  fn context(self, context: impl Into<String>) -> Self::Output {
    self.map_err(|error| {
      MelodyError::FileError(ContextualError::new(error.into(), context.into()))
    })
  }
}

impl<T, FE> Contextualize for Result<T, singlefile::UserError<FE, MelodyError>>
where singlefile::Error<FE>: Into<MelodyFileError> {
  type Output = MelodyResult<T>;

  fn context(self, context: impl Into<String>) -> Self::Output {
    self.map_err(|error| {
      MelodyError::FileError(ContextualError::new(
        match error {
          singlefile::UserError::Format(error) => singlefile::Error::Format(error).into(),
          singlefile::UserError::Io(error) => singlefile::Error::Io(error).into(),
          singlefile::UserError::User(error) => return error
        },
        context.into()
      ))
    })
  }
}

impl<T> Contextualize for Result<T, cleverbot_logs::Error> {
  type Output = MelodyResult<T>;

  fn context(self, context: impl Into<String>) -> Self::Output {
    self.map_err(|error| {
      MelodyError::FileError(ContextualError::new(MelodyFileError::CleverBotLog(error), context.into()))
    })
  }
}

impl<T> Contextualize for Result<T, serenity::Error> {
  type Output = MelodyResult<T>;

  fn context(self, context: impl Into<String>) -> Self::Output {
    self.map_err(|error| MelodyError::SerenityError(ContextualError::new(error, context.into())))
  }
}

#[macro_export]
macro_rules! operate {
  ($core:expr, $function:ident::<$Key:ty>, $operation:expr) => {
    $crate::data::$function($core.get::<$Key>().await, $operation).await
  };
}
