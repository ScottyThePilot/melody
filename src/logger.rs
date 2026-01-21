use chrono::Local;
use fern::{Dispatch, Output, InitError};
use fern::colors::{Color, ColoredLevelConfig};
use flate2::Compression;
use flate2::write::GzEncoder;
use fs_err::{File, OpenOptions};
use log::LevelFilter;

use std::fmt;
use std::io::{self, BufReader, BufWriter};
use std::path::PathBuf;
use std::sync::mpsc::Sender;



// 8 MiB
const LOG_FILE_SIZE_LIMIT: u64 = 8 * 1024 * 1024;

pub fn setup(sender: Sender<String>) -> Result<(), InitError> {
  let me = env!("CARGO_PKG_NAME").replace('-', "_");

  let colors = ColoredLevelConfig::new()
    .error(Color::Red)
    .warn(Color::Yellow)
    .info(Color::White)
    .debug(Color::BrightBlue)
    .trace(Color::BrightBlack);

  Dispatch::new()
    .format(move |out, message, record| {
      out.finish(format_args!(
        "{time}[{level}]({target}) {message}",
        time = Local::now().format("[%H:%M:%S]"),
        level = record.level(),
        target = record.target(),
        message = message
      ))
    })
    .level(LevelFilter::Warn)
    .level_for(me, LevelFilter::Trace)
    .level_for("feed_machine", LevelFilter::Debug)
    .level_for("melody_commander", LevelFilter::Info)
    .level_for("melody_flag", LevelFilter::Info)
    .level_for("melody_framework", LevelFilter::Trace)
    .level_for("melody_ratelimiter", LevelFilter::Info)
    .level_for("melody_rss_feed", LevelFilter::Info)
    .chain({
      Output::call(move |record| {
        let color = colors.get_color(&record.level());
        let line = format!("{}\n", WithFgColor::new(record.args(), color));
        sender.send(line).expect("error while logging");
      })
    })
    .chain({
      fs_err::create_dir_all("./data/")?;
      let file = create_or_rotate_log_file()?;
      Output::writer(Box::new(BufWriter::new(file)), "\n")
    })
    .apply()?;
  Ok(())
}

fn create_file(path: impl Into<PathBuf>, create: bool) -> io::Result<File> {
  OpenOptions::new().create(create).truncate(create).read(true).write(true).open(path)
}

fn create_append_file(path: impl Into<PathBuf>) -> io::Result<File> {
  OpenOptions::new().create(true).append(true).truncate(false).open(path)
}

fn create_or_rotate_log_file() -> io::Result<File> {
  fs_err::create_dir_all("./data/logs/")?;

  let file = create_append_file("./data/logs/latest.log")?;

  let metadata = file.metadata()?;
  if metadata.len() >= LOG_FILE_SIZE_LIMIT {
    let now = Local::now().format("%Y-%m-%d-%H-%M-%S");
    let out_path = format!("./data/logs/{now}.log.gz");

    // copy the contents of ./data/logs/latest.log to ./data/logs/{now}.log.gz, with compression
    let mut reader = BufReader::new(create_file("./data/logs/latest.log", false)?);
    let mut writer = GzEncoder::new(BufWriter::new(create_file(&out_path, true)?), Compression::new(6));
    io::copy(&mut reader, &mut writer)?;

    // clear the contents of ./data/logs/latest.log
    reader.get_ref().set_len(0)?;
  };

  Ok(file)
}



#[derive(Debug)]
pub struct WithFgColor<T> {
  text: T,
  color: Color,
}

impl<T: fmt::Display> WithFgColor<T> {
  pub const fn new(text: T, color: Color) -> Self {
    WithFgColor { text, color }
  }
}

impl<T> fmt::Display for WithFgColor<T>
where T: fmt::Display {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "\x1B[{}m", self.color.to_fg_str())?;
    fmt::Display::fmt(&self.text, f)?;
    write!(f, "\x1B[0m")?;
    Ok(())
  }
}
