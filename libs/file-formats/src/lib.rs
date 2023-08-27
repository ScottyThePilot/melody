extern crate ciborium;
extern crate serde;
extern crate singlefile;
#[macro_use]
extern crate thiserror;
extern crate toml;
extern crate xz2;

use serde::ser::Serialize;
use serde::de::DeserializeOwned;
use singlefile::manager::FileFormat;
use xz2::write::XzEncoder;
use xz2::read::XzDecoder;

use std::io::{Read, Write};



#[derive(Debug, Error)]
pub enum TomlError {
  #[error(transparent)]
  Io(#[from] std::io::Error),
  #[error(transparent)]
  De(#[from] toml::de::Error),
  #[error(transparent)]
  Ser(#[from] toml::ser::Error)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Toml;

impl<T> FileFormat<T> for Toml
where T: Serialize + DeserializeOwned {
  type FormatError = TomlError;

  fn from_reader<R: Read>(&self, mut reader: R) -> Result<T, Self::FormatError> {
    let mut buf = String::new();
    reader.read_to_string(&mut buf)?;
    toml::from_str(&buf).map_err(From::from)
  }

  fn to_writer<W: Write>(&self, mut writer: W, value: &T) -> Result<(), Self::FormatError> {
    let buf = toml::to_string_pretty(value)?;
    writer.write_all(buf.as_bytes())?;
    Ok(())
  }

  fn to_buffer(&self, value: &T) -> Result<Vec<u8>, Self::FormatError> {
    match toml::to_string_pretty(value) {
      Ok(buf) => Ok(buf.into_bytes()),
      Err(error) => Err(error.into())
    }
  }
}



#[derive(Debug, Error)]
pub enum CborError {
  #[error(transparent)]
  De(#[from] ciborium::de::Error<std::io::Error>),
  #[error(transparent)]
  Ser(#[from] ciborium::ser::Error<std::io::Error>)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Cbor;

impl<T> FileFormat<T> for Cbor
where T: Serialize + DeserializeOwned {
  type FormatError = CborError;

  fn from_reader<R: Read>(&self, reader: R) -> Result<T, Self::FormatError> {
    ciborium::from_reader(reader).map_err(From::from)
  }

  fn to_writer<W: Write>(&self, writer: W, value: &T) -> Result<(), Self::FormatError> {
    ciborium::into_writer(value, writer).map_err(From::from)
  }
}



pub type XzCbor = XzCompressed<Cbor>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct XzCompressed<F>(pub F);

impl<T, F> FileFormat<T> for XzCompressed<F>
where T: Serialize + DeserializeOwned, F: FileFormat<T> {
  type FormatError = <F as FileFormat<T>>::FormatError;

  fn from_reader<R: Read>(&self, reader: R) -> Result<T, Self::FormatError> {
    self.0.from_reader(XzDecoder::new(reader))
  }

  fn to_writer<W: Write>(&self, writer: W, value: &T) -> Result<(), Self::FormatError> {
    self.0.to_writer(XzEncoder::new(writer, 9), value)
  }
}
