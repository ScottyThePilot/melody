use ahash::AHasher;
use once_cell::sync::Lazy;

use std::env::current_exe;
use std::hash::Hasher;
use std::fs::File;
use std::io;



static BUILD_ID: Lazy<u64> = Lazy::new(|| {
  let mut hasher = HashWriter(AHasher::default());
  let mut binary = current_exe()
    .and_then(File::open)
    .map(io::BufReader::new)
    .unwrap();
  io::copy(&mut binary, &mut hasher).unwrap();
  let build_id = hasher.0.finish();
	trace!("Build ID: {build_id}");
	build_id
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
