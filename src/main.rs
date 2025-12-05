#![warn(
  absolute_paths_not_starting_with_crate,
  redundant_imports,
  redundant_lifetimes,
  future_incompatible,
  deprecated_in_future,
  missing_copy_implementations,
  missing_debug_implementations
)]

extern crate ahash;
extern crate build_info;
extern crate cacheable;
extern crate chrono;
extern crate chumsky;
extern crate cleverbot;
extern crate cleverbot_logs;
extern crate const_random;
extern crate defy;
extern crate feed_machine;
extern crate fern;
extern crate float_ord;
extern crate fs_err;
extern crate futures;
extern crate ids;
extern crate itertools;
#[macro_use]
extern crate log;
extern crate melody_commander;
extern crate melody_connect_four;
extern crate melody_flag;
extern crate melody_framework;
extern crate melody_ratelimiter;
extern crate poise;
extern crate rand;
extern crate regex;
extern crate reqwest;
#[macro_use]
extern crate serde;
extern crate serenity;
extern crate singlefile;
extern crate singlefile_formats;
extern crate songbird;
extern crate symphonia;
extern crate term_stratum;
#[macro_use]
extern crate thiserror;
extern crate tokio;
extern crate tracing;
extern crate uord;
extern crate url;
extern crate yggdrasil;

#[macro_use] pub(crate) mod utils;
pub(crate) mod prelude;
pub(crate) mod commands;
pub(crate) mod data;
pub(crate) mod feature;
pub(crate) mod handler;

use serenity::prelude::SerenityError;
use term_stratum::StratumEvent;
use tokio::sync::mpsc::UnboundedReceiver as AsyncReceiver;

use std::sync::mpsc::Sender as SyncSender;

pub const BUILD_ID: u64 = const_random::const_random!(u64);
pub const BUILD_DATE: &str = build_info::build_datetime_local_fixed_format!("%-d %b %Y %-H:%M:%S %z");
pub const BUILD_GIT_HASH: &str = build_info::git_hash_short!();

fn main() {
  yggdrasil::reroot().expect("unable to set root dir");
  let (logger_sender, logger_receiver) = std::sync::mpsc::channel();
  let (event_sender, event_receiver) = tokio::sync::mpsc::unbounded_channel();
  setup_logger(logger_sender).expect("failed to setup logger");

  term_stratum::run(
    env!("CARGO_PKG_NAME"),
    logger_receiver,
    move || run(event_receiver),
    move |event| event_sender.send(event).unwrap()
  );
}

#[tokio::main]
async fn run(event_reciever: AsyncReceiver<StratumEvent>) {
  match crate::handler::launch(event_reciever).await {
    Err(error) => error!("{error}"),
    Ok(()) => ()
  };
}

fn setup_logger(sender: SyncSender<String>) -> Result<(), fern::InitError> {
  let me = env!("CARGO_PKG_NAME").replace('-', "_");
  fern::Dispatch::new()
    .format(move |out, message, record| {
      out.finish(format_args!(
        "{}[{}]({}) {}",
        chrono::Local::now().format("[%H:%M:%S]"),
        record.level(),
        record.target(),
        message
      ))
    })
    .level(log::LevelFilter::Warn)
    .level_for(me, log::LevelFilter::Trace)
    .level_for("feed_machine", log::LevelFilter::Trace)
    .level_for("melody_commander", log::LevelFilter::Info)
    .level_for("melody_flag", log::LevelFilter::Info)
    .level_for("melody_framework", log::LevelFilter::Trace)
    .level_for("melody_ratelimiter", log::LevelFilter::Info)
    .level_for("melody_rss_feed", log::LevelFilter::Info)
    .chain(sender)
    .chain({
      fs_err::create_dir_all("./data/")?;
      fern::log_file("./data/latest.log")?
    })
    .apply()?;
  Ok(())
}

pub type MelodyResult<T = ()> = Result<T, MelodyError>;

/// An error that can occur during operation of the bot.
#[derive(Debug, Error)]
pub enum MelodyError {
  #[error("File Error: {0}")]
  FileError(#[from] defy::ContextualError<MelodyFileError>),
  #[error("Serenity Error: {0}")]
  SerenityError(#[from] defy::ContextualError<SerenityError>),
  #[error("Command Error: {0}")]
  CommandError(#[from] MelodyCommandError),
  #[error("Input Error: {0}")]
  InputError(#[from] crate::melody_commander::CommandError),
  #[error("YT-DLP Error: {0}")]
  YtDlpError(#[from] crate::utils::youtube::YtDlpError),
  #[error("Feed Model Error: {0}")]
  FeedModelError(#[from] feed_machine::model::ModelError<feed_machine::model::SchemaError>)
}

impl MelodyError {
  pub const COMMAND_NOT_IN_GUILD: Self = MelodyError::CommandError(MelodyCommandError::NOT_IN_GUILD);
  pub const COMMAND_PRECONDITION_VIOLATION_ARGUMENTS: Self = MelodyError::CommandError(MelodyCommandError::PRECONDITION_VIOLATION_ARGUMENTS);
  pub const COMMAND_PRECONDITION_VIOLATION_ROOT_COMMAND: Self = MelodyError::CommandError(MelodyCommandError::PRECONDITION_VIOLATION_ROOT_COMMAND);

  pub const fn command_cache_failure(message: &'static str) -> Self {
    MelodyError::CommandError(MelodyCommandError::cache_failure(message))
  }

  pub const fn command_precondition_violation(message: &'static str) -> Self {
    MelodyError::CommandError(MelodyCommandError::precondition_violation(message))
  }
}

/// An error that can be caused by trying to read or parse a file.
#[derive(Debug, Error)]
pub enum MelodyFileError {
  #[error(transparent)]
  Io(#[from] std::io::Error),
  #[error(transparent)]
  Toml(#[from] singlefile::Error<singlefile_formats::data::toml_serde::TomlError>),
  #[error(transparent)]
  Json(#[from] singlefile::Error<singlefile_formats::data::json_serde::JsonError>),
  #[error(transparent)]
  Cbor(#[from] singlefile::Error<singlefile_formats::data::cbor_serde::CborError>),
  #[error(transparent)]
  CleverBotLog(#[from] cleverbot_logs::Error)
}

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MelodyCommandError {
  #[error("not in a guild")]
  NotInGuild,
  #[error("data not cached: {0}")]
  CacheFailure(&'static str),
  #[error("precondition violated: {0}")]
  PreconditionViolation(&'static str)
}

impl MelodyCommandError {
  pub const NOT_IN_GUILD: Self = MelodyCommandError::NotInGuild;
  pub const PRECONDITION_VIOLATION_ARGUMENTS: Self = MelodyCommandError::precondition_violation("arguments");
  pub const PRECONDITION_VIOLATION_ROOT_COMMAND: Self = MelodyCommandError::precondition_violation("root command");

  pub const fn cache_failure(message: &'static str) -> Self {
    Self::CacheFailure(message)
  }

  pub const fn precondition_violation(message: &'static str) -> Self {
    Self::PreconditionViolation(message)
  }
}
