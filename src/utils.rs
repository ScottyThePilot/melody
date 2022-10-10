use crate::{MelodyError, MelodyResult};

use chrono::{DateTime, Utc};
use itertools::Itertools;

use std::error::Error;
use std::fmt;

pub fn capitalize(s: impl AsRef<str>) -> String {
  let mut chars = s.as_ref().chars();
  chars.next().map_or_else(String::new, |first| {
    first.to_uppercase()
      .chain(chars.map(|c| c.to_ascii_lowercase()))
      .collect()
  })
}

pub fn capitalize_words(s: impl AsRef<str>) -> String {
  s.as_ref().split("-").map(capitalize).join(" ")
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

  fn context(self, context: &'static str) -> Self::Output;
}

impl<T> Contextualize for Result<T, singlefile::Error> {
  type Output = MelodyResult<T>;

  fn context(self, context: &'static str) -> Self::Output {
    self.map_err(|error| MelodyError::FileError(error, context))
  }
}

impl<T> Contextualize for Result<T, serenity::Error> {
  type Output = MelodyResult<T>;

  fn context(self, context: &'static str) -> Self::Output {
    self.map_err(|error| MelodyError::SerenityError(error, context))
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
