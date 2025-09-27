#![allow(dead_code)]

const FOLDER_POOL_SIZE: usize = 32;
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

#[derive(Clone, Copy)]
pub struct FolderEntry {
    name: [u8; 32],
    name_len: usize,
    exists: bool,
    files: [FileEntry; 8],
    subfolders: [*mut FolderEntry; 4], 
}

static mut FOLDER_POOL: [FolderEntry; FOLDER_POOL_SIZE] = [FolderEntry {
    name: [0; 32],
    name_len: 0,
    exists: false,
    files: [FileEntry {
        name: [0; 32],
        name_len: 0,
        data: [0; 512],
        data_len: 0,
        exists: false,
    }; 8],
    subfolders: [core::ptr::null_mut(); 4],
}; 32];

static mut FOLDER_POOL_INDEX: usize = 0;

impl FolderEntry {
    pub const fn new() -> Self {
        Self {
            name: [0; 32],
            name_len: 0,
            exists: false,
            files: [FileEntry::new(); 8],
            subfolders: [core::ptr::null_mut(); 4],
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
    fn get_name(&self) -> &[u8] {
        &self.name[..self.name_len]
    }

    fn find_subfolder(&self, name: &str) -> Option<usize> {
        for (index, folder) in self.subfolders.iter().enumerate() {
            if !folder.is_null() {
                unsafe {
                    if let Some(folder_ref) = unsafe { folder.as_ref() } {
                        if folder_ref.exists && folder_ref.name_matches(name) {
                            return Some(index);
                        }
                    }
                }
            }
        }
        None
    }

    fn find_free_subfolder_slot(&self) -> Option<usize> {
        for (index, folder) in self.subfolders.iter().enumerate() {
            if folder.is_null() {
                return Some(index);
            }
        }
        None
    }

    fn add_subfolder(&mut self, name: &str) -> Result<(), FileSystemError> {
        for slot in self.subfolders.iter_mut() {
            if slot.is_null() {
                unsafe {
                    if FOLDER_POOL_INDEX >= FOLDER_POOL_SIZE {
                        return Err(FileSystemError::DiskFull);
                    }
                    let new_folder = &raw mut FOLDER_POOL[FOLDER_POOL_INDEX] as *mut FolderEntry;
                    FOLDER_POOL_INDEX += 1;

                    (*new_folder).set_name(name)?;
                    (*new_folder).exists = true;
                    *slot = new_folder;
                }
                return Ok(());
            }
        }
        Err(FileSystemError::DiskFull)
    }

    fn remove_subfolder(&mut self, name: &str) -> Result<(), FileSystemError> {
        for slot in self.subfolders.iter_mut() {
            if !slot.is_null() {
                unsafe {
                    if (*(*slot)).name_matches(name) {

                        core::ptr::drop_in_place(*slot);
                        *slot = core::ptr::null_mut();
                        return Ok(());
                    }
                }
            }
        }
        Err(FileSystemError::FileNotFound)
    }
}

impl FileEntry {
    pub const fn new() -> Self {
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
    folders: [*mut FolderEntry; 4],
    current_dir: [*mut FolderEntry; 8], 
    current_dir_depth: usize, 
}

impl OsFileSystem {
    pub const fn new() -> Self {
        Self {
            files: [FileEntry::new(); 8],
            folders: [core::ptr::null_mut(); 4],
            current_dir: [core::ptr::null_mut(); 8],
            current_dir_depth: 0,
        }
    }
}

static mut GLOBAL_FS: OsFileSystem = OsFileSystem::new();

impl Drop for OsFileSystem {
    fn drop(&mut self) {
        for folder in self.folders.iter_mut() {
            if !folder.is_null() {
                unsafe {
                    core::ptr::drop_in_place(*folder);
                }
                *folder = core::ptr::null_mut();
            }
        }
    }
}

impl OsFileSystem {
    pub fn change_directory(&mut self, path: &str) -> Result<(), FileSystemError> {
        if path == ".." {
            if self.current_dir_depth > 0 {
                self.current_dir_depth -= 1;
                return Ok(());
            }
            return Ok(()); 
        }

        let target_folder = if self.current_dir_depth == 0 {

            if let Some(index) = self.folders.iter().position(|&f| !f.is_null() && unsafe { (*f).name_matches(path) }) {
                unsafe { self.folders[index] }
            } else {
                return Err(FileSystemError::FileNotFound);
            }
        } else {

            let current = self.current_dir[self.current_dir_depth - 1];
            if let Some(index) = unsafe { (*current).find_subfolder(path) } {
                unsafe { (*current).subfolders[index] }
            } else {
                return Err(FileSystemError::FileNotFound);
            }
        };

        if self.current_dir_depth >= self.current_dir.len() {
            return Err(FileSystemError::InvalidPath);
        }

        self.current_dir[self.current_dir_depth] = target_folder;
        self.current_dir_depth += 1;
        Ok(())
    }

    pub fn list_current_directory(&self) -> ([Option<&[u8]>; 4], [Option<&[u8]>; 8]) {
        let mut folders = [None; 4];
        let mut files = [None; 8];
        let mut folder_count = 0;

        if self.current_dir_depth == 0 {

            for (i, &folder) in self.folders.iter().enumerate() {
                if !folder.is_null() {
                    unsafe {
                        if (*folder).exists {
                            if folder_count < folders.len() {
                                folders[folder_count] = Some((*folder).get_name());
                                folder_count += 1;
                            }
                        }
                    }
                }
            }
        } else {

            let current = unsafe { &*self.current_dir[self.current_dir_depth - 1] };
            for (i, &subfolder) in current.subfolders.iter().enumerate() {
                if !subfolder.is_null() {
                    unsafe {
                        if (*subfolder).exists {
                            if folder_count < folders.len() {
                                folders[folder_count] = Some((*subfolder).get_name());
                                folder_count += 1;
                            }
                        }
                    }
                }
            }
        }

        (folders, files)
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

    pub fn create_folder(&mut self, path: &str) -> Result<(), FileSystemError> {
        let mut parts = [""; 8];
        let mut part_count = 0;
        for part in path.split('/') {
            if part_count >= parts.len() {
                return Err(FileSystemError::InvalidPath);
            }
            parts[part_count] = part;
            part_count += 1;
        }

        let mut current_folder: Option<*mut FolderEntry> = None;
        for &part in &parts[..part_count] {
            unsafe {
                if let Some(folder_ptr) = current_folder {
                    if let Some(index) = (*folder_ptr).find_subfolder(part) {
                        current_folder = Some((*folder_ptr).subfolders[index]);
                    } else {
                        (*folder_ptr).add_subfolder(part)?;
                        current_folder = (*folder_ptr)
                            .subfolders
                            .iter()
                            .find(|&&f| !f.is_null())
                            .copied();
                    }
                } else {
                    if let Some(index) = self.folders.iter().position(|&f| !f.is_null() && (*f).name_matches(part)) {
                        current_folder = Some(self.folders[index]);
                    } else {
                        if let Some(slot_index) = self.folders.iter().position(|&f| f.is_null()) {
                            if FOLDER_POOL_INDEX >= FOLDER_POOL_SIZE {
                                return Err(FileSystemError::DiskFull);
                            }
                            let new_folder = &raw mut FOLDER_POOL[FOLDER_POOL_INDEX] as *mut FolderEntry;
                            FOLDER_POOL_INDEX += 1;

                            (*new_folder).set_name(part)?;
                            (*new_folder).exists = true;
                            self.folders[slot_index] = new_folder;
                            current_folder = Some(new_folder);
                        } else {
                            return Err(FileSystemError::DiskFull);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn delete_folder(&mut self, path: &str) -> Result<(), FileSystemError> {
        let mut parts = [""; 8];
        let mut part_count = 0;
        for part in path.split('/') {
            if part_count >= parts.len() {
                return Err(FileSystemError::InvalidPath);
            }
            parts[part_count] = part;
            part_count += 1;
        }

        let mut current_folder: Option<*mut FolderEntry> = None;
        let mut parent_folder: Option<*mut FolderEntry> = None;
        let mut folder_name = "";

        for &part in &parts[..part_count] {
            folder_name = part;
            unsafe {
                if let Some(folder_ptr) = current_folder {
                    parent_folder = Some(folder_ptr);
                    if let Some(index) = (*folder_ptr).find_subfolder(part) {
                        current_folder = Some((*folder_ptr).subfolders[index]);
                    } else {
                        return Err(FileSystemError::FileNotFound);
                    }
                } else {
                    if let Some(index) = self.folders.iter().position(|&f| !f.is_null() && (*f).name_matches(part)) {
                        parent_folder = None;
                        current_folder = Some(self.folders[index]);
                    } else {
                        return Err(FileSystemError::FileNotFound);
                    }
                }
            }
        }

        unsafe {
            if let Some(parent_ptr) = parent_folder {
                (*parent_ptr).remove_subfolder(folder_name)
            } else if let Some(index) = self.folders.iter().position(|&f| !f.is_null() && (*f).name_matches(folder_name)) {
                core::ptr::drop_in_place(self.folders[index]);
                self.folders[index] = core::ptr::null_mut();
                Ok(())
            } else {
                Err(FileSystemError::FileNotFound)
            }
        }
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
