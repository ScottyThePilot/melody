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
  FileError(MelodyFileError, String),
  #[error("Serenity Error: {1} ({0})")]
  SerenityError(serenity::Error, String),
  #[error("Invalid command")]
  InvalidCommand,
  #[error("Invalid arguments")]
  InvalidArguments
}

#[derive(Debug, Error)]
pub enum MelodyFileError {
  #[error(transparent)]
  Io(#[from] std::io::Error),
  #[error(transparent)]
  Toml(#[from] singlefile::Error<crate::data::TomlError>),
  #[error(transparent)]
  Cbor(#[from] singlefile::Error<crate::data::CborError>)
}

fn setup_logger() -> Result<(), fern::InitError> {
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
    .level_for("melody", log::LevelFilter::Trace)
    .chain(std::io::stdout())
    .chain({
      std::fs::create_dir_all("./data/")?;
      fern::log_file("./data/latest.log")?
    })
    .apply()?;
  Ok(())
}
