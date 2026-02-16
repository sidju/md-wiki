use std::cell::RefCell;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Trait for file system operations to allow dependency injection and mocking
pub trait FileSystem {
    /// Read a file to a string
    fn read_to_string(&self, path: &Path) -> Result<String, Box<dyn Error>>;
    
    /// Write a string to a file
    fn write(&self, path: &Path, contents: &str) -> Result<(), Box<dyn Error>>;
    
    /// Create a directory and all parent directories if they don't exist
    fn create_dir_all(&self, path: &Path) -> Result<(), Box<dyn Error>>;
    
    /// Copy a file from source to destination
    fn copy(&self, from: &Path, to: &Path) -> Result<(), Box<dyn Error>>;
    
    /// Check if a path exists
    fn exists(&self, path: &Path) -> bool;
    
    /// Walk a directory and return all file paths
    fn walk_dir(&self, path: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>>;
}

/// Real file system implementation
pub struct RealFileSystem;

impl FileSystem for RealFileSystem {
    fn read_to_string(&self, path: &Path) -> Result<String, Box<dyn Error>> {
        Ok(fs::read_to_string(path)?)
    }
    
    fn write(&self, path: &Path, contents: &str) -> Result<(), Box<dyn Error>> {
        Ok(fs::write(path, contents)?)
    }
    
    fn create_dir_all(&self, path: &Path) -> Result<(), Box<dyn Error>> {
        Ok(fs::create_dir_all(path)?)
    }
    
    fn copy(&self, from: &Path, to: &Path) -> Result<(), Box<dyn Error>> {
        fs::copy(from, to)?;
        Ok(())
    }
    
    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }
    
    fn walk_dir(&self, path: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
        let mut paths = Vec::new();
        for entry in WalkDir::new(path) {
            let entry = entry?;
            if entry.path().is_file() {
                paths.push(entry.path().to_path_buf());
            }
        }
        Ok(paths)
    }
}

/// Mock file system for testing
pub struct MockFileSystem {
    files: RefCell<HashMap<PathBuf, String>>,
}

impl MockFileSystem {
    pub fn new() -> Self {
        Self {
            files: RefCell::new(HashMap::new()),
        }
    }
    
    pub fn add_file(&self, path: impl Into<PathBuf>, content: impl Into<String>) {
        self.files.borrow_mut().insert(path.into(), content.into());
    }
    
    pub fn get_file(&self, path: &Path) -> Option<String> {
        self.files.borrow().get(path).cloned()
    }
}

impl Default for MockFileSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl FileSystem for MockFileSystem {
    fn read_to_string(&self, path: &Path) -> Result<String, Box<dyn Error>> {
        self.files
            .borrow()
            .get(path)
            .cloned()
            .ok_or_else(|| format!("File not found: {:?}", path).into())
    }
    
    fn write(&self, path: &Path, contents: &str) -> Result<(), Box<dyn Error>> {
        self.files.borrow_mut().insert(path.to_path_buf(), contents.to_string());
        Ok(())
    }
    
    fn create_dir_all(&self, _path: &Path) -> Result<(), Box<dyn Error>> {
        // No-op for mock
        Ok(())
    }
    
    fn copy(&self, from: &Path, to: &Path) -> Result<(), Box<dyn Error>> {
        let content = self.read_to_string(from)?;
        self.write(to, &content)?;
        Ok(())
    }
    
    fn exists(&self, path: &Path) -> bool {
        self.files.borrow().contains_key(path)
    }
    
    fn walk_dir(&self, base_path: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
        let mut paths: Vec<PathBuf> = self.files
            .borrow()
            .keys()
            .filter(|path| path.starts_with(base_path))
            .cloned()
            .collect();
        // Sort to ensure deterministic order (like WalkDir does)
        paths.sort();
        Ok(paths)
    }
}
