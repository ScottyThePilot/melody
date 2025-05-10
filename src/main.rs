#![warn(missing_debug_implementations)]
extern crate ahash;
extern crate chrono;
extern crate chumsky;
extern crate cleverbot;
extern crate cleverbot_logs;
extern crate const_random;
extern crate dunce;
extern crate fern;
extern crate float_ord;
extern crate ids;
extern crate itertools;
#[macro_use]
extern crate log;
extern crate melody_commander;
extern crate melody_flag;
extern crate melody_rss_feed;
extern crate once_cell;
extern crate rand;
extern crate regex;
#[macro_use]
extern crate serde;
extern crate serenity;
extern crate singlefile;
extern crate singlefile_formats;
#[macro_use]
extern crate thiserror;
extern crate tokio;
extern crate uord;
extern crate url;
extern crate yggdrasil;

#[macro_use] pub(crate) mod utils;
#[macro_use] pub(crate) mod blueprint;
pub(crate) mod prelude;
pub(crate) mod commands;
pub(crate) mod data;
pub(crate) mod feature;
pub(crate) mod handler;
pub(crate) mod ratelimiter;

use serenity::prelude::SerenityError;
use serenity::model::id::GenericId;
use term_stratum::StratumEvent;
use tokio::sync::mpsc::UnboundedReceiver as AsyncReceiver;

use std::sync::mpsc::Sender as SyncSender;

pub const BUILD_ID: u64 = const_random::const_random!(u64);

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
  #[error("File Error: {1} ({0})")]
  FileError(MelodyFileError, String),
  #[error("Serenity Error: {1} ({0})")]
  SerenityError(SerenityError, String),
  #[error("Command Error: {0}")]
  CommandError(MelodyCommandError),
  #[error("Input Error: {0}")]
  InputError(#[from] crate::melody_commander::CommandError),
  #[error("YT-DLP Error: {0}")]
  YtDlpError(#[from] crate::utils::youtube::YtDlpError)
}

impl MelodyError {
  pub const COMMAND_NOT_IN_GUILD: Self = MelodyError::CommandError(MelodyCommandError::NotInGuild);
  pub const COMMAND_INVALID_ARGUMENTS_STRUCTURE: Self = MelodyError::CommandError(MelodyCommandError::InvalidArgumentsStructure);

  pub const fn command_cache_failure(message: &'static str) -> Self {
    MelodyError::CommandError(MelodyCommandError::CacheFailure(message))
  }
}

impl From<MelodyParseCommandError> for MelodyError {
  fn from(error: MelodyParseCommandError) -> Self {
    MelodyError::CommandError(MelodyCommandError::FailedToParse(error))
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

#[derive(Debug, Error, Clone)]
pub enum MelodyCommandError {
  #[error("not in a guild")]
  NotInGuild,
  #[error("failed to parse interaction: {0}")]
  FailedToParse(#[from] MelodyParseCommandError),
  #[error("invalid arguments structure")]
  InvalidArgumentsStructure,
  #[error("invalid arguments: {0}")]
  InvalidArguments(String),
  #[error("data not cached: {0}")]
  CacheFailure(&'static str)
}

#[derive(Debug, Error, Clone, Copy)]
pub enum MelodyParseCommandError {
  #[error("no command found")]
  NoCommandFound,
  #[error("unresolved mentionable (generic) id")]
  UnresolvedGenericId(GenericId),
  #[error("invalid structure")]
  InvalidStructure
}
