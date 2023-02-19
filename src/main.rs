#![warn(missing_debug_implementations)]
#[macro_use] extern crate log;
#[macro_use] extern crate serde;
#[macro_use] extern crate thiserror;
extern crate ahash;
extern crate chrono;
extern crate command_attr;
extern crate fern;
extern crate float_ord;
extern crate itertools;
extern crate once_cell;
extern crate rand;
extern crate serenity;
extern crate serde_cbor;
extern crate singlefile;
extern crate tokio;
extern crate toml;
extern crate xz2;

#[macro_use] pub(crate) mod utils;
#[macro_use] pub(crate) mod blueprint;
pub(crate) mod build_id;
pub(crate) mod commands;
pub(crate) mod data;
pub(crate) mod feature;
pub(crate) mod handler;
pub(crate) mod ratelimiter;
pub(crate) mod terminal;

use crate::terminal::Flag;

use serenity::prelude::SerenityError;
use tokio::sync::Mutex;
use tokio::sync::mpsc::{unbounded_channel as mpsc_channel, UnboundedReceiver as MpscReceiver};
use tokio::sync::oneshot::{channel as oneshot_channel, Receiver as OneshotReceiver};

use std::sync::Arc;

fn main() {
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
  pub const COMMAND_FAILED_TO_PARSE: Self = MelodyError::CommandError(MelodyCommandError::FailedToParse);
  pub const COMMAND_INVALID_ARGUMENTS_STRUCTURE: Self = MelodyError::CommandError(MelodyCommandError::InvalidArgumentsStructure);

  pub const fn command_cache_failure(message: &'static str) -> Self {
    MelodyError::CommandError(MelodyCommandError::CacheFailure(message))
  }
}

/// An error that can be caused by trying to read or parse a file.
#[derive(Debug, Error)]
pub enum MelodyFileError {
  #[error(transparent)]
  Io(#[from] std::io::Error),
  #[error(transparent)]
  Toml(#[from] singlefile::Error<crate::data::TomlError>),
  #[error(transparent)]
  Cbor(#[from] singlefile::Error<crate::data::CborError>)
}

#[derive(Debug, Error, Clone)]
pub enum MelodyCommandError {
  #[error("not in a guild")]
  NotInGuild,
  #[error("failed to parse interaction")]
  FailedToParse,
  #[error("invalid arguments structure")]
  InvalidArgumentsStructure,
  #[error("invalid arguments: {0}")]
  InvalidArguments(String),
  #[error("data not cached: {0}")]
  CacheFailure(&'static str)
}
