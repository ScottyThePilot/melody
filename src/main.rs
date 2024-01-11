#![warn(missing_debug_implementations)]
extern crate ahash;
extern crate chrono;
extern crate chumsky;
extern crate cleverbot;
extern crate cleverbot_logs;
extern crate commander;
extern crate command_attr;
extern crate const_random;
extern crate dunce;
extern crate fern;
extern crate float_ord;
extern crate ids;
extern crate itertools;
extern crate linefeed;
#[macro_use]
extern crate log;
extern crate once_cell;
extern crate rand;
extern crate regex;
extern crate rss_feed;
#[macro_use]
extern crate serde;
extern crate serenity;
extern crate singlefile;
extern crate singlefile_formats;
#[macro_use]
extern crate thiserror;
extern crate tokio;
extern crate uord;

#[macro_use] pub(crate) mod utils;
#[macro_use] pub(crate) mod blueprint;
pub(crate) mod commands;
pub(crate) mod data;
pub(crate) mod feature;
pub(crate) mod handler;
pub(crate) mod ratelimiter;
pub(crate) mod terminal;

use crate::terminal::Flag;

use serenity::prelude::SerenityError;
use serenity::model::id::GenericId;
use tokio::sync::Mutex;
use tokio::sync::mpsc::{unbounded_channel as mpsc_channel, UnboundedReceiver as MpscReceiver};
use tokio::sync::oneshot::{channel as oneshot_channel, Receiver as OneshotReceiver};

use std::sync::Arc;

pub const BUILD_ID: u64 = const_random::const_random!(u64);

fn main() {
  let root = crate::utils::root_dir().expect("unable to find root dir");
  std::env::set_current_dir(root).expect("unable to set root dir");

  let (terminate_sender, terminate_receiver) = oneshot_channel();
  let (input_sender, input_receiver) = mpsc_channel();
  crate::terminal::run(
    // Main task to be run for the duration of the terminal
    move |kill_flag| run(kill_flag, terminate_receiver, input_receiver),
    // One-time code to be executed when the terminate signal is sent from the terminal
    move |kill_flag| terminate_sender.send(kill_flag).unwrap(),
    // Code to be executed when input is sent from the terminal
    move |line| input_sender.send(line).unwrap()
  );

  println!();
}

#[tokio::main]
async fn run(kill_flag: Flag, terminate: OneshotReceiver<Flag>, input: MpscReceiver<String>) {
  let terminate = Arc::new(Mutex::new(terminate));
  let input = Arc::new(Mutex::new(input));

  loop {
    match crate::handler::launch(terminate.clone(), input.clone()).await {
      Err(error) => return error!("{error}"),
      Ok(true) if !kill_flag.get() => continue,
      Ok(..) => break
    };
  };
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
  CommandError(MelodyCommandError)
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
  Toml(#[from] singlefile::Error<singlefile_formats::toml_serde::TomlError>),
  #[error(transparent)]
  Cbor(#[from] singlefile::Error<singlefile_formats::cbor_serde::CborError>),
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
