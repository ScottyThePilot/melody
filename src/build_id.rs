use ahash::random_state::RandomState;
use once_cell::sync::Lazy;

use std::env::current_exe;
use std::hash::{BuildHasher, Hasher};
use std::fs::File;
use std::io;



static BUILD_ID: Lazy<u64> = Lazy::new(|| -> u64 {
  let hasher = RandomState::with_seeds(0, 0, 0, 0)
    .build_hasher();
  let mut writer = HashWriter(hasher);
  let mut reader = current_exe()
    .and_then(File::open)
    .map(io::BufReader::new)
    .unwrap();
  io::copy(&mut reader, &mut writer).unwrap();
  writer.0.finish()
});

pub fn get() -> u64 {
  *BUILD_ID
}

#[repr(transparent)]
struct HashWriter<T: Hasher>(T);

impl<T: Hasher> io::Write for HashWriter<T> {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    self.0.write(buf);
    Ok(buf.len())
  }

  fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
    self.0.write(buf);
    Ok(())
  }

  fn flush(&mut self) -> io::Result<()> {
    Ok(())
  }
}
