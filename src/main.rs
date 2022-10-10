#![warn(missing_debug_implementations)]
extern crate ahash;
extern crate chrono;
extern crate command_attr;
extern crate fern;
#[macro_use]
extern crate log;
extern crate once_cell;
extern crate serenity;
extern crate singlefile;
#[macro_use]
extern crate serde;
extern crate serde_cbor;
#[macro_use]
extern crate thiserror;
extern crate tokio;
extern crate toml;

#[macro_use]
pub(crate) mod utils;
#[macro_use]
pub(crate) mod blueprint;
pub(crate) mod build_id;
pub(crate) mod commands;
pub(crate) mod data;
pub(crate) mod feature;
pub(crate) mod handler;

use fern::Dispatch;

#[tokio::main]
async fn main() {
  setup_logger().unwrap();

  loop {
    match crate::handler::launch().await {
      Ok(true) => continue,
      Ok(false) => break,
      Err(error) => return error!("{error}")
    };
  };
}

pub type MelodyResult<T = ()> = Result<T, MelodyError>;

#[derive(Debug, Error)]
pub enum MelodyError {
  #[error("File Error: {1} ({0})")]
  FileError(singlefile::Error, &'static str),
  #[error("Serenity Error: {1} ({0})")]
  SerenityError(serenity::Error, &'static str),
  #[error("Invalid command")]
  InvalidCommand,
  #[error("Invalid arguments")]
  InvalidArguments
}

fn setup_logger() -> Result<(), fern::InitError> {
  Dispatch::new()
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
    .level_for("melody", log::LevelFilter::Trace)
    .chain(std::io::stdout())
    .chain({
      std::fs::create_dir_all("./data/")?;
      fern::log_file("./data/latest.log")?
    })
    .apply()?;
  Ok(())
}
