#![warn(
  absolute_paths_not_starting_with_crate,
  redundant_imports,
  redundant_lifetimes,
  future_incompatible,
  deprecated_in_future,
  missing_copy_implementations,
  missing_debug_implementations
)]

extern crate abyss;
extern crate ahash;
extern crate build_info;
extern crate chrono;
extern crate cleverbot;
extern crate cleverbot_logs;
extern crate const_random;
extern crate feed_machine;
extern crate fern;
extern crate float_ord;
extern crate fs_err;
extern crate futures;
extern crate itertools;
#[macro_use]
extern crate log;
extern crate melody_commander;
extern crate melody_connect_four;
extern crate melody_flag;
extern crate melody_framework;
extern crate melody_random;
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

#[allow(unused)]
macro_rules! println {
  ($($tt:tt)*) => (info!($($tt)*));
}

pub(crate) mod commands;
pub(crate) mod data;
pub(crate) mod error;
pub(crate) mod feature;
pub(crate) mod handler;
pub(crate) mod logger;
pub(crate) mod prelude;
pub(crate) mod utils;

use term_stratum::StratumEvent;
use tokio::sync::mpsc::UnboundedReceiver as AsyncReceiver;

pub const BUILD_ID: u64 = const_random::const_random!(u64);
pub const BUILD_DATE: &str = build_info::build_datetime_local_fixed_format!("%-d %b %Y %-H:%M:%S %z");
pub const BUILD_GIT_HASH: &str = build_info::git_hash_short!();

fn main() {
  abyss::sys::set_current_dir_to_exe_dir().expect("unable to set root dir");
  let (logger_sender, logger_receiver) = std::sync::mpsc::channel();
  let (event_sender, event_receiver) = tokio::sync::mpsc::unbounded_channel();
  crate::logger::setup(logger_sender).expect("failed to setup logger");

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
