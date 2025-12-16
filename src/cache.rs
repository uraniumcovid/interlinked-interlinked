use crate::FileIndex;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheData {
    pub index: FileIndex,
    pub scan_time: u64,
    pub directory: String,
    pub file_mtimes: HashMap<String, u64>,
}

pub struct Cache {
    cache_dir: PathBuf,
}

impl Cache {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let cache_dir = if let Some(proj_dirs) = ProjectDirs::from("", "", "interlinked-interlinked") {
            proj_dirs.data_dir().to_path_buf()
        } else {
            // Fallback for systems without XDG support
            dirs::home_dir()
                .ok_or("Could not find home directory")?
                .join(".local/share/interlinked-interlinked")
        };
        
        fs::create_dir_all(&cache_dir)?;
        
        Ok(Self { cache_dir })
    }

    pub fn get_cache_path(&self, directory: &str) -> PathBuf {
        // Create a safe filename from the directory path
        let safe_name = directory
            .replace('/', "_")
            .replace('\\', "_")
            .replace(':', "_")
            .replace("~", "home");
        
        self.cache_dir.join(format!("{}.json", safe_name))
    }

    pub fn load(&self, directory: &str) -> Result<Option<FileIndex>, Box<dyn std::error::Error>> {
        let cache_path = self.get_cache_path(directory);
        
        if !cache_path.exists() {
            return Ok(None);
        }

        let cache_data: CacheData = serde_json::from_str(&fs::read_to_string(&cache_path)?)?;
        
        // Check if cache is for the same directory
        if cache_data.directory != directory {
            return Ok(None);
        }

        // Check if any files have been modified since cache was created
        if self.is_cache_stale(&cache_data, directory)? {
            return Ok(None);
        }

        Ok(Some(cache_data.index))
    }

    pub fn save(&self, index: &FileIndex, directory: &str, file_mtimes: HashMap<String, u64>) -> Result<(), Box<dyn std::error::Error>> {
        let cache_data = CacheData {
            index: index.clone(),
            scan_time: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            directory: directory.to_string(),
            file_mtimes,
        };

        let cache_path = self.get_cache_path(directory);
        let json = serde_json::to_string_pretty(&cache_data)?;
        fs::write(&cache_path, json)?;

        println!("Cache saved to: {}", cache_path.display());
        Ok(())
    }

    pub fn clear(&self, directory: &str) -> Result<(), Box<dyn std::error::Error>> {
        let cache_path = self.get_cache_path(directory);
        if cache_path.exists() {
            fs::remove_file(&cache_path)?;
            println!("Cache cleared: {}", cache_path.display());
        }
        Ok(())
    }

    pub fn info(&self, directory: &str) -> Result<(), Box<dyn std::error::Error>> {
        let cache_path = self.get_cache_path(directory);
        
        if !cache_path.exists() {
            println!("No cache found for directory: {}", directory);
            return Ok(());
        }

        let cache_data: CacheData = serde_json::from_str(&fs::read_to_string(&cache_path)?)?;
        let cache_time = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(cache_data.scan_time);
        
        println!("Cache Info:");
        println!("  Directory: {}", cache_data.directory);
        println!("  Cache file: {}", cache_path.display());
        println!("  Created: {:?}", cache_time);
        println!("  Files tracked: {}", cache_data.file_mtimes.len());
        println!("  Links: {}", cache_data.index.backlinks.len());
        println!("  Tags: {}", cache_data.index.tags.len());
        println!("  Is stale: {}", self.is_cache_stale(&cache_data, directory)?);

        Ok(())
    }

    fn is_cache_stale(&self, cache_data: &CacheData, directory: &str) -> Result<bool, Box<dyn std::error::Error>> {
        // Quick check: if any tracked file has a different mtime, cache is stale
        for (file_path, cached_mtime) in &cache_data.file_mtimes {
            if let Ok(metadata) = fs::metadata(file_path) {
                if let Ok(modified) = metadata.modified() {
                    let current_mtime = modified.duration_since(UNIX_EPOCH)?.as_secs();
                    if current_mtime != *cached_mtime {
                        return Ok(true);
                    }
                }
            } else {
                // File no longer exists, cache is stale
                return Ok(true);
            }
        }

        // Check if new files have been added (this is more expensive)
        // For now, we'll do a simple directory mtime check
        if let Ok(metadata) = fs::metadata(directory) {
            if let Ok(modified) = metadata.modified() {
                let dir_mtime = modified.duration_since(UNIX_EPOCH)?.as_secs();
                if dir_mtime > cache_data.scan_time {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    pub fn collect_file_mtimes<P: AsRef<Path>>(&self, directory: P) -> Result<HashMap<String, u64>, Box<dyn std::error::Error>> {
        let mut mtimes = HashMap::new();
        
        for entry in walkdir::WalkDir::new(directory) {
            let entry = entry?;
            if entry.file_type().is_file() {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        let mtime = modified.duration_since(UNIX_EPOCH)?.as_secs();
                        mtimes.insert(entry.path().to_string_lossy().to_string(), mtime);
                    }
                }
            }
        }
        
        Ok(mtimes)
    }
}

impl Clone for FileIndex {
    fn clone(&self) -> Self {
        Self {
            links: self.links.clone(),
            backlinks: self.backlinks.clone(),
            tags: self.tags.clone(),
            file_tags: self.file_tags.clone(),
        }
    }
}