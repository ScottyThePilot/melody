extern crate chrono;
extern crate flate2;
extern crate fs_err;
extern crate log;

use abyss::logging::{OutputMut, OutputWrite};
use abyss::externs::bytemuck::TransparentWrapper;
use chrono::{DateTime, Local};
use flate2::Compression;
use flate2::write::GzEncoder;
use fs_err::File;
use log::trace;

use std::io;
use std::io::prelude::*;
use std::path::PathBuf;
use std::time::Duration;



#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileRecyclableOptions {
  pub parent_dir: PathBuf,
  pub filename_latest_format: String,
  pub filename_recycled_format: String,
  pub created_age_limit: Duration,
  pub size_limit: u64
}

#[derive(Debug)]
struct FileInner {
  file: File,
  created: DateTime<Local>,
  size_current: u64
}

impl FileInner {
  fn new(path: impl Into<PathBuf>) -> io::Result<Self> {
    let file = File::options()
      .create(true).append(true)
      .write(true).read(true)
      .truncate(false)
      .open(path)?;

    let metadata = file.metadata()?;
    let created = DateTime::from(metadata.created()?);
    let size_current = metadata.len();

    Ok(FileInner { file, created, size_current })
  }

  fn is_stale(&self, now: DateTime<Local>, options: &FileRecyclableOptions) -> bool {
    let created_age = now.signed_duration_since(self.created).to_std().unwrap_or(Duration::ZERO);
    self.size_current > options.size_limit || created_age > options.created_age_limit
  }

  fn into_file(mut self) -> io::Result<File> {
    self.file.seek(io::SeekFrom::Start(0))?;
    Ok(self.file)
  }
}

impl Write for FileInner {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    let bytes_written = self.file.write(buf)?;
    self.size_current += bytes_written as u64;
    Ok(bytes_written)
  }

  fn write_vectored(&mut self, bufs: &[io::IoSlice<'_>]) -> io::Result<usize> {
    let bytes_written = self.file.write_vectored(bufs)?;
    self.size_current += bytes_written as u64;
    Ok(bytes_written)
  }

  fn flush(&mut self) -> io::Result<()> {
    self.file.flush()
  }
}

#[derive(Debug)]
pub struct FileRecyclable {
  options: FileRecyclableOptions,
  inner: Option<FileInner>
}

impl FileRecyclable {
  fn inner(&mut self) -> io::Result<&mut FileInner> {
    let now = Local::now();
    try_get_or_insert_with(&mut self.inner, || {
      let latest_format = now.format(&self.options.filename_latest_format);
      let latest_path = self.options.parent_dir.join(latest_format.to_string());
      FileInner::new(latest_path)
    })
  }

  fn maybe_recycle(&mut self) -> io::Result<bool> {
    let now = Local::now();
    if let Some(inner) = self.inner.take_if(|inner| inner.is_stale(now, &self.options)) {
      let created = inner.created;
      let mut file = inner.into_file()?;

      let recycled_format = created.format(&self.options.filename_recycled_format);
      let recycled_path = self.options.parent_dir.join(recycled_format.to_string());

      let mut recycled_file = GzEncoder::new(File::create(&recycled_path)?, Compression::new(7));
      io::copy(&mut file, &mut recycled_file)?;

      let path = file.into_path();

      fs_err::remove_file(&path)?;

      // this is done on another thread to avoid any deadlocks,
      // since we are a logger that's potentially calling ourselves
      std::thread::spawn(move || {
        trace!("recycled {} to {}", path.display(), recycled_path.display());
      });

      Ok(true)
    } else {
      Ok(false)
    }
  }

  pub fn new(options: FileRecyclableOptions) -> Self {
    FileRecyclable { options, inner: None }
  }

  pub fn write_with(&mut self, f: impl FnOnce(&mut (dyn Write + Send + Sync)) -> io::Result<()>) -> io::Result<()> {
    f(self.inner()?)?;
    self.maybe_recycle()?;
    Ok(())
  }

  pub fn write_bytes(&mut self, buf: &[u8]) -> io::Result<()> {
    self.write_with(|writer| writer.write_all(buf))?;
    Ok(())
  }

  pub fn write_record(&mut self, record: &log::Record) -> io::Result<()> {
    self.write_with(|writer| {
      OutputWrite::wrap_mut(writer).log(record)
    })?;
    Ok(())
  }

  pub fn flush(&mut self) -> io::Result<()> {
    self.write_with(|writer| {
      OutputWrite::wrap_mut(writer).flush()
    })?;
    self.inner()?;
    Ok(())
  }
}

impl OutputMut for FileRecyclable {
  type Error = io::Error;

  fn enabled(&mut self, _: &log::Metadata) -> bool {
    true
  }

  fn log(&mut self, record: &log::Record) -> io::Result<()> {
    self.write_record(record)
  }

  fn flush(&mut self) -> io::Result<()> {
    self.flush()
  }
}



fn try_get_or_insert_with<T, E, F>(option: &mut Option<T>, f: F) -> Result<&mut T, E>
where F: FnOnce() -> Result<T, E> {
  match option {
    Some(value) => Ok(value),
    None => Ok(option.insert(f()?))
  }
}
