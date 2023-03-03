use crate::{MelodyError, MelodyFileError, MelodyResult};

use chrono::{DateTime, Utc};
use itertools::Itertools;
use rand::SeedableRng;
use rand::rngs::SmallRng;
use serenity::model::id::UserId;

use std::error::Error;
use std::fmt;

pub fn create_rng() -> SmallRng {
  SmallRng::from_rng(rand::thread_rng()).expect("failed to seed smallrng")
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

/// Takes message content and a user ID, returning the remainder of the message if the string
/// mentioned the user at the start of it.
pub fn strip_user_mention(msg: &str, user_id: UserId) -> Option<&str> {
  let msg = msg.trim().strip_prefix("<@")?;
  let msg = msg.strip_prefix("!").unwrap_or(msg);
  let msg = msg.strip_prefix(&user_id.to_string())?;
  let msg = msg.strip_prefix(">")?.trim_start();
  Some(msg)
}

#[derive(Debug, Clone, Copy)]
pub struct Blockify<S>(pub S);

impl<S: fmt::Display> Blockify<S> {
  pub fn new(s: S) -> Self {
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
  pub fn new(timestamp: DateTime<Utc>, format: TimestampFormat) -> Self {
    Timestamp(timestamp, format)
  }
}

impl fmt::Display for Timestamp {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<t:{}:{}>", self.0.timestamp(), self.1)
  }
}

pub struct List<L>(pub L);

impl<L> fmt::Display for List<L>
where L: IntoIterator + Copy, L::Item: fmt::Display {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut touched = false;
    for (i, item) in self.0.into_iter().enumerate() {
      if i != 0 { f.write_str(", ")? };
      fmt::Display::fmt(&item, f)?;
      touched = true;
    };

    if !touched {
      f.write_str("(none)")?;
    };

    Ok(())
  }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum TimestampFormat {
  ShortTime,
  LongTime,
  ShortDate,
  LongDate,
  FullDateShortTime,
  FullDateLongTime,
  Relative
}

impl fmt::Display for TimestampFormat {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.write_str(match self {
      TimestampFormat::ShortTime => "t",
      TimestampFormat::LongTime => "T",
      TimestampFormat::ShortDate => "d",
      TimestampFormat::LongDate => "D",
      TimestampFormat::FullDateShortTime => "f",
      TimestampFormat::FullDateLongTime => "F",
      TimestampFormat::Relative => "R"
    })
  }
}

pub trait Contextualize {
  type Output;

  fn context(self, context: impl Into<String>) -> Self::Output;

  fn context_log(self, context: impl Into<String>)
  where Self: Sized, Self::Output: Loggable {
    self.context(context).log()
  }
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

impl<T> Contextualize for Result<T, serenity::Error> {
  type Output = MelodyResult<T>;

  fn context(self, context: impl Into<String>) -> Self::Output {
    self.map_err(|error| MelodyError::SerenityError(error, context.into()))
  }
}

pub trait Loggable {
  type Ok;
  type Err: Error;

  fn log(self);
  fn log_some(self) -> Option<Self::Ok>;
}

impl<T, E: Error> Loggable for Result<T, E> {
  type Ok = T;
  type Err = E;

  fn log(self) {
    if let Err(error) = self {
      error!("{error}");
    };
  }

  fn log_some(self) -> Option<T> {
    match self {
      Ok(value) => Some(value),
      Err(error) => {
        error!("{error}");
        None
      }
    }
  }
}

#[macro_export]
macro_rules! log_return {
  ($expr:expr) => ($crate::some_or_return!($expr.log_some()));
}

#[macro_export]
macro_rules! ok_or_continue {
  ($expr:expr) => (match $expr {
    Result::Ok(value) => value,
    Result::Err(_) => continue
  });
}

#[macro_export]
macro_rules! some_or_return {
  ($expr:expr) => (match $expr {
    Option::Some(value) => value,
    Option::None => return
  });
}

#[macro_export]
macro_rules! some_or_continue {
  ($expr:expr) => (match $expr {
    Option::Some(value) => value,
    Option::None => continue
  });
}
