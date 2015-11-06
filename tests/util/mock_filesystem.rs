// use std::slice;
use std::path::{Path, PathBuf};
use std::io;
use std::io::{Read, Write};
use std::collections::hash_map::{HashMap, Entry};
use std::sync::{Arc, Mutex, MutexGuard};
use std::cmp;

use rex::filesystem::FileSystem;

use super::bytes;

lazy_static! {
    pub static ref FILES: Mutex<HashMap<PathBuf, Arc<Mutex<Vec<u8>>>>> = Mutex::new(HashMap::new());
}

pub struct MockFile {
    pub locked_vec: Arc<Mutex<Vec<u8>>>,
    pub pos: u64,
}

impl MockFile {
    fn new(vec: Arc<Mutex<Vec<u8>>>) -> MockFile {
        MockFile {
            locked_vec: vec,
            pos: 0,
        }
    }
}

impl Read for MockFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let vec = self.locked_vec.lock().unwrap();
        let len = cmp::min(vec.len() - self.pos as usize, buf.len());
        let src_slice = &vec[self.pos as usize..(self.pos as usize + len)];
        {
            let mut dest_slice = &mut buf[..len];


            assert_eq!(src_slice.len(), dest_slice.len());

            bytes::copy_memory(src_slice, dest_slice);
        }
        self.pos += len as u64;

        Ok(len)
    }
}

impl Write for MockFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut vec = self.locked_vec.lock().unwrap();

        // Since we do not support seek, this is pretty simple
        vec.extend(buf.iter());

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub struct MockFileSystem;


impl FileSystem for MockFileSystem {
    type FSRead = MockFile;
    type FSWrite = MockFile;

    fn open<P: AsRef<Path>>(path: P) -> io::Result<Self::FSRead> {
        FILES.lock().unwrap().get(path.as_ref()).ok_or(io::Error::new(io::ErrorKind::NotFound, "File not found!")).map(|file|
            MockFile::new(file.clone())
        )
    }

    fn save<P: AsRef<Path>>(path: P) -> io::Result<Self::FSWrite> {
        let file = Arc::new(Mutex::new(Vec::new()));
        if let Entry::Vacant(entry) = FILES.lock().unwrap().entry(path.as_ref().into()) {
            entry.insert(file.clone());
            Ok(MockFile::new(file))
        } else {
            Err(io::Error::new(io::ErrorKind::AlreadyExists, "File alredy exists!"))
        }
    }
}

impl MockFileSystem {
    pub fn get_inner<'a, P: AsRef<Path>>(path: P) -> Vec<u8> {
        // This function is very ugly, in general we would like to "unwrap" the file from the
        // mock filesystem. Sadly, there doesn't seem to be a better way.
        let f = FILES.lock().unwrap().remove(path.as_ref()).unwrap();
        let a = Arc::try_unwrap(f).unwrap();
        let m = a.lock().unwrap();
        let v = m.clone();
        v
    }

    pub fn put<'a, P: AsRef<Path>>(path: P, v: Vec<u8>) {
        FILES.lock().unwrap().insert(path.as_ref().into(), Arc::new(Mutex::new(v)));
    }
}
