#![no_std]
#![allow(dead_code)]

#[derive(Debug)]
pub enum FileSystemError {
    FileNotFound,
    ReadError,
    WriteError,
    UnknownError,
    InvalidPath,
    InvalidRoot,
    PermissionDenied,
    DiskFull,
    NotADirectory,
}

pub struct OsFileSystem;

const MAX_FILE_SIZE: usize = 256;
const MAX_PATH_SIZE: usize = 64; 

static mut FILE_DATA: [u8; MAX_FILE_SIZE] = [0; MAX_FILE_SIZE];
static mut FILE_LEN: usize = 0;
static mut HAS_FILE: bool = false;
static mut FILE_PATH: [u8; MAX_PATH_SIZE] = [0; MAX_PATH_SIZE];
static mut FILE_PATH_LEN: usize = 0;

impl OsFileSystem {
    pub fn new() -> Self {
        OsFileSystem
    }

    pub fn write_file(&mut self, path: &str, data: &[u8]) -> Result<(), FileSystemError> {
        let path_bytes = path.as_bytes();
        if path_bytes.len() > MAX_PATH_SIZE {
            return Err(FileSystemError::InvalidPath);
        }
        if data.len() > MAX_FILE_SIZE {
            return Err(FileSystemError::DiskFull);
        }

        unsafe {

            FILE_DATA[..data.len()].copy_from_slice(data);
            FILE_LEN = data.len();

            FILE_PATH[..path_bytes.len()].copy_from_slice(path_bytes);
            FILE_PATH_LEN = path_bytes.len();

            HAS_FILE = true;
        }
        Ok(())
    }

    fn path_matches(&self, path: &str) -> bool {
        unsafe {
            let provided_path_bytes = path.as_bytes();
            if provided_path_bytes.len() != FILE_PATH_LEN {
                return false;
            }

            for i in 0..FILE_PATH_LEN {
                if provided_path_bytes[i] != FILE_PATH[i] {
                    return false;
                }
            }
            true
        }
    }

    pub fn read_file(&self, path: &str) -> Result<&'static [u8], FileSystemError> {
        unsafe {
            if HAS_FILE && self.path_matches(path) {
                return Ok(&FILE_DATA[..FILE_LEN]);
            }
            Err(FileSystemError::FileNotFound)
        }
    }

    pub fn delete_file(&mut self, path: &str) -> Result<(), FileSystemError> {
        unsafe {
            if HAS_FILE && self.path_matches(path) {
                FILE_LEN = 0;
                FILE_PATH_LEN = 0;
                HAS_FILE = false;
                return Ok(());
            }
            Err(FileSystemError::FileNotFound)
        }
    }

    pub fn list_files(&self) -> Result<Option<&'static [u8]>, FileSystemError> {
        unsafe {
            if HAS_FILE {

                Ok(Some(&FILE_PATH[..FILE_PATH_LEN]))
            } else {
                Ok(None)
            }
        }
    }
}

pub fn new_os_file_system() -> OsFileSystem {
    OsFileSystem::new()
}
