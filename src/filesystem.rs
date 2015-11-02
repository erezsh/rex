use std::fs::File;
use std::io;
use std::io::Read;
use std::io::Write;
use std::path::Path;

pub trait FileSystem {
  type FSRead: Read;
  type FSWrite: Write;
  fn open<P: AsRef<Path>>(p: P) -> io::Result<Self::FSRead>;
  fn save<P: AsRef<Path>>(p: P) -> io::Result<Self::FSWrite>;
}

pub struct DefaultFileSystem;
impl FileSystem for DefaultFileSystem {
    type FSRead = File;
    type FSWrite = File;

    fn open<P: AsRef<Path>>(p: P) -> io::Result<Self::FSRead> {
        File::open(p)
    }
    fn save<P: AsRef<Path>>(p: P) -> io::Result<Self::FSWrite> {
        File::create(p)
    }
}
