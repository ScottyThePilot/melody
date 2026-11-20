use abyss::logging::{LevelsConfig, OutputExt, OutputMapError, FormatterCallback, Rich};
use chrono::Local;
use colored::{Color, Colorize};
use melody_logger::{FileRecyclable, FileRecyclableOptions};
use log::{LevelFilter, SetLoggerError};

use std::fmt;
use std::io;
use std::path::PathBuf;
use std::sync::mpsc::{Sender, SendError};
use std::sync::Mutex;
use std::time::Duration;



#[derive(Debug, Error)]
pub enum LogError {
  #[error("failed to send log record: {0}")]
  SenderError(#[from] SendError<String>),
  #[error("io error: {0}")]
  IoError(#[from] io::Error),
  #[error(transparent)]
  SetLoggerError(#[from] SetLoggerError)
}

const LOG_FILE_SIZE_LIMIT: u64 = crate::utils::convert::mib(8);
const LOG_FILE_CREATED_AGE_LIMIT: Duration = Duration::from_hours(24);

const LEVELS_CONFIG_COLORS: LevelsConfig<Color> = LevelsConfig {
  error: Color::Red,
  warn: Color::Yellow,
  info: Color::White,
  debug: Color::BrightBlue,
  trace: Color::BrightBlack
};

pub fn setup(sender: Sender<String>) -> Result<(), LogError> {
  log::set_max_level(LevelFilter::Trace);

  fs_err::create_dir_all("./data/logs/")?;

  let file_recyclable = FileRecyclable::new(FileRecyclableOptions {
    parent_dir: PathBuf::from("./data/logs/"),
    filename_latest_format: "latest.log".to_owned(),
    filename_recycled_format: "%Y-%m-%d-%H-%M-%S.log.gz".to_owned(),
    created_age_limit: LOG_FILE_CREATED_AGE_LIMIT,
    size_limit: LOG_FILE_SIZE_LIMIT
  });

  let me = env!("CARGO_PKG_NAME").replace('-', "_");

  Rich::<LogError>::new()
    .with_format(move |callback, message, record| {
      callback.finish(format_args!(
        "{time}[{level}]({target}) {message}",
        time = Local::now().format("[%H:%M:%S]"),
        level = record.level(),
        target = record.target()
      ))
    })
    .with_level(LevelFilter::Warn)
    .with_level_for(me, LevelFilter::Trace)
    .with_level_for("feed_machine", LevelFilter::Debug)
    .with_level_for("melody_commander", LevelFilter::Info)
    .with_level_for("melody_flag", LevelFilter::Info)
    .with_level_for("melody_framework", LevelFilter::Trace)
    .with_level_for("melody_logger", LevelFilter::Trace)
    .with_level_for("melody_ratelimiter", LevelFilter::Info)
    .with_level_for("melody_rss_feed", LevelFilter::Info)
    .with_child({
      Rich::<LogError>::new()
        .with_format(move |callback, message, record| {
          colorize(callback, LEVELS_CONFIG_COLORS[record.level()], *message)
        })
        .with_child(OutputMapError::new(sender, LogError::SenderError))
    })
    .with_child(OutputMapError::new(Mutex::new(file_recyclable), LogError::IoError))
    .apply()?;

  Ok(())
}

fn colorize<E>(callback: FormatterCallback<'_, E>, color: Color, message: fmt::Arguments) -> Result<(), E> {
  if colored::control::SHOULD_COLORIZE.should_colorize() {
    callback.finish(format_args!("{}", message.to_string().color(color)))
  } else {
    callback.finish(message)
  }
}
