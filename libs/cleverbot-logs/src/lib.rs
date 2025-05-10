extern crate base64;
pub extern crate chrono;
extern crate ciborium;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate thiserror;

use base64::engine::general_purpose::STANDARD;
use chrono::{DateTime, Utc};

use std::io::{self, BufRead, BufReader, BufWriter, Read, Write};
use std::path::Path;

use base64::read::DecoderReader;
use base64::write::EncoderWriter;
use fs_err::File;

#[derive(Debug)]
pub struct CleverBotLogger {
  file: File
}

impl CleverBotLogger {
  pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
    File::options().append(true)
      .open(path.as_ref()).map(|file| CleverBotLogger { file })
  }

  pub fn create<P: AsRef<Path>>(path: P) -> io::Result<Self> {
    File::options().append(true).create(true)
      .open(path.as_ref()).map(|file| CleverBotLogger { file })
  }

  pub fn log(&self, entry: &CleverBotLogEntry) -> Result<(), Error> {
    encode_entries(&self.file, std::slice::from_ref(entry))
  }
}



#[derive(Debug, Error)]
pub enum Error {
  #[error(transparent)]
  Io(#[from] io::Error),
  #[error(transparent)]
  Deserialize(#[from] ciborium::de::Error<io::Error>),
  #[error(transparent)]
  Serialize(#[from] ciborium::ser::Error<io::Error>)
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CleverBotLogEntry {
  pub thread: u64,
  #[serde(with = "chrono::serde::ts_milliseconds")]
  pub time: DateTime<Utc>,
  pub message: String,
  pub response: String
}

impl CleverBotLogEntry {
  pub fn encode<W: Write>(&self, writer: W) -> Result<(), Error> {
    ciborium::into_writer(self, EncoderWriter::new(writer, &STANDARD))
      .map_err(Error::Serialize)
  }

  pub fn decode<R: Read>(reader: R) -> Result<Self, Error> {
    ciborium::from_reader(DecoderReader::new(reader, &STANDARD))
      .map_err(Error::Deserialize)
  }
}

pub fn encode_entries<W: Write>(writer: W, entries: &[CleverBotLogEntry]) -> Result<(), Error> {
  let mut writer = BufWriter::new(writer);
  for entry in entries {
    entry.encode(&mut writer)?;
    writeln!(&mut writer)?;
  };

  writer.flush()?;
  Ok(())
}

pub fn decode_entries_buffered<R: Read>(reader: R) -> Result<Vec<CleverBotLogEntry>, Error> {
  decode_entries(BufReader::new(reader))
}

pub fn decode_entries<R: BufRead>(reader: R) -> Result<Vec<CleverBotLogEntry>, Error> {
  reader.lines()
    .map(|result| result.map_err(Error::Io))
    .map(|result| result.and_then(|string| CleverBotLogEntry::decode(string.as_bytes())))
    .collect::<Result<Vec<CleverBotLogEntry>, Error>>()
}
