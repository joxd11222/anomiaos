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

#[derive(Clone, Copy)]
pub struct FileEntry {
    name: [u8; 32],      
    name_len: usize,     
    data: [u8; 512],     
    data_len: usize,     
    exists: bool,        
}

impl FileEntry {
    fn new() -> Self {
        Self {
            name: [0; 32],
            name_len: 0,
            data: [0; 512],
            data_len: 0,
            exists: false,
        }
    }

    fn name_matches(&self, path: &str) -> bool {
        let path_bytes = path.as_bytes();
        if path_bytes.len() != self.name_len {
            return false;
        }

        for i in 0..self.name_len {
            if path_bytes[i] != self.name[i] {
                return false;
            }
        }
        true
    }

    fn set_name(&mut self, path: &str) -> Result<(), FileSystemError> {
        let path_bytes = path.as_bytes();
        if path_bytes.len() > self.name.len() {
            return Err(FileSystemError::InvalidPath);
        }

        self.name[..path_bytes.len()].copy_from_slice(path_bytes);
        self.name_len = path_bytes.len();
        Ok(())
    }

    fn set_data(&mut self, data: &[u8]) -> Result<(), FileSystemError> {
        if data.len() > self.data.len() {
            return Err(FileSystemError::DiskFull);
        }

        self.data[..data.len()].copy_from_slice(data);
        self.data_len = data.len();
        Ok(())
    }

    fn get_name(&self) -> &[u8] {
        &self.name[..self.name_len]
    }

    fn get_data(&self) -> &[u8] {
        &self.data[..self.data_len]
    }
}

pub struct OsFileSystem {
    files: [FileEntry; 8], 
}

static mut GLOBAL_FS: OsFileSystem = OsFileSystem {
    files: [FileEntry {
        name: [0; 32],
        name_len: 0,
        data: [0; 512],
        data_len: 0,
        exists: false,
    }; 8],
};

impl OsFileSystem {
    pub fn new() -> Self {
        OsFileSystem {
            files: [FileEntry::new(); 8],
        }
    }

    fn find_file(&self, path: &str) -> Option<usize> {
        for (index, file) in self.files.iter().enumerate() {
            if file.exists && file.name_matches(path) {
                return Some(index);
            }
        }
        None
    }

    fn find_free_slot(&self) -> Option<usize> {
        for (index, file) in self.files.iter().enumerate() {
            if !file.exists {
                return Some(index);
            }
        }
        None
    }

    pub fn write_file(&mut self, path: &str, data: &[u8]) -> Result<(), FileSystemError> {

        if let Some(index) = self.find_file(path) {

            self.files[index].set_data(data)?;
            return Ok(());
        }

        if let Some(index) = self.find_free_slot() {
            self.files[index].set_name(path)?;
            self.files[index].set_data(data)?;
            self.files[index].exists = true;
            Ok(())
        } else {
            Err(FileSystemError::DiskFull)
        }
    }

    pub fn read_file(&self, path: &str) -> Result<&[u8], FileSystemError> {
        if let Some(index) = self.find_file(path) {
            Ok(self.files[index].get_data())
        } else {
            Err(FileSystemError::FileNotFound)
        }
    }

    pub fn delete_file(&mut self, path: &str) -> Result<(), FileSystemError> {
        if let Some(index) = self.find_file(path) {
            self.files[index].exists = false;
            self.files[index].name_len = 0;
            self.files[index].data_len = 0;
            Ok(())
        } else {
            Err(FileSystemError::FileNotFound)
        }
    }

    pub fn list_files(&self) -> Result<Option<&[u8]>, FileSystemError> {

        for file in &self.files {
            if file.exists {
                return Ok(Some(file.get_name()));
            }
        }
        Ok(None)
    }

    pub fn list_all_files(&self) -> [Option<&[u8]>; 8] {
        let mut result = [None; 8];
        for (i, file) in self.files.iter().enumerate() {
            if file.exists {
                result[i] = Some(file.get_name());
            }
        }
        result
    }
}

pub fn new_os_file_system() -> *mut OsFileSystem {
    unsafe { &raw mut GLOBAL_FS }
}

pub fn with_fs<F, R>(f: F) -> R 
where
    F: FnOnce(&OsFileSystem) -> R,
{
    unsafe {
        let fs_ptr = new_os_file_system();
        f(&*fs_ptr)
    }
}

pub fn with_fs_mut<F, R>(f: F) -> R 
where
    F: FnOnce(&mut OsFileSystem) -> R,
{
    unsafe {
        let fs_ptr = new_os_file_system();
        f(&mut *fs_ptr)
    }
}
