use clap::Parser;
use directories::ProjectDirs;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub scan_all_text: bool,
    pub file_extensions: Vec<String>,
    pub binary_extensions: Vec<String>,
    pub ignore_patterns: Vec<String>,
    pub link_pattern: String,
    pub tag_pattern: String,
    pub output_format: OutputFormat,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum OutputFormat {
    Pretty,
    Json,
    Compact,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            scan_all_text: true,
            file_extensions: vec![
                "txt".to_string(),
                "md".to_string(),
                "markdown".to_string(),
                "org".to_string(),
                "rst".to_string(),
                "adoc".to_string(),
                "asciidoc".to_string(),
                "tex".to_string(),
                "typ".to_string(),
                "py".to_string(),
                "rs".to_string(),
                "js".to_string(),
                "ts".to_string(),
                "html".to_string(),
                "css".to_string(),
                "json".to_string(),
                "yaml".to_string(),
                "yml".to_string(),
                "toml".to_string(),
                "xml".to_string(),
                "csv".to_string(),
            ],
            binary_extensions: vec![
                "png".to_string(),
                "jpg".to_string(),
                "jpeg".to_string(),
                "gif".to_string(),
                "bmp".to_string(),
                "svg".to_string(),
                "ico".to_string(),
                "webp".to_string(),
                "mp4".to_string(),
                "avi".to_string(),
                "mov".to_string(),
                "mp3".to_string(),
                "wav".to_string(),
                "flac".to_string(),
                "pdf".to_string(),
                "doc".to_string(),
                "docx".to_string(),
                "xls".to_string(),
                "xlsx".to_string(),
                "ppt".to_string(),
                "pptx".to_string(),
                "zip".to_string(),
                "tar".to_string(),
                "gz".to_string(),
                "rar".to_string(),
                "7z".to_string(),
                "exe".to_string(),
                "dll".to_string(),
                "so".to_string(),
                "dylib".to_string(),
                "bin".to_string(),
                "iso".to_string(),
                "dmg".to_string(),
                "pkg".to_string(),
                "deb".to_string(),
                "rpm".to_string(),
                "woff".to_string(),
                "woff2".to_string(),
                "ttf".to_string(),
                "otf".to_string(),
                "eot".to_string(),
            ],
            ignore_patterns: vec![
                ".*".to_string(),
                "target".to_string(),
                "node_modules".to_string(),
                ".git".to_string(),
                "dist".to_string(),
                "build".to_string(),
                "__pycache__".to_string(),
                ".next".to_string(),
                ".nuxt".to_string(),
                "vendor".to_string(),
            ],
            link_pattern: r"\[\[([^\]]+)\]\]".to_string(),
            tag_pattern: r"(?m)^tags:\s*(.+)$".to_string(),
            output_format: OutputFormat::Pretty,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileIndex {
    pub links: HashMap<String, Vec<String>>,
    pub backlinks: HashMap<String, Vec<String>>,
    pub tags: HashMap<String, Vec<String>>,
    pub file_tags: HashMap<String, Vec<String>>,
}

impl FileIndex {
    pub fn new() -> Self {
        Self {
            links: HashMap::new(),
            backlinks: HashMap::new(),
            tags: HashMap::new(),
            file_tags: HashMap::new(),
        }
    }

    pub fn add_link(&mut self, source_file: &str, target: &str) {
        self.links
            .entry(source_file.to_string())
            .or_insert_with(Vec::new)
            .push(target.to_string());
        
        self.backlinks
            .entry(target.to_string())
            .or_insert_with(Vec::new)
            .push(source_file.to_string());
    }

    pub fn add_tag(&mut self, file: &str, tag: &str) {
        self.file_tags
            .entry(file.to_string())
            .or_insert_with(Vec::new)
            .push(tag.to_string());
        
        self.tags
            .entry(tag.to_string())
            .or_insert_with(Vec::new)
            .push(file.to_string());
    }
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        if let Some(proj_dirs) = ProjectDirs::from("", "", "interlinked") {
            let config_path = proj_dirs.config_dir().join("config.toml");
            
            if config_path.exists() {
                let content = fs::read_to_string(&config_path)?;
                let config: Config = toml::from_str(&content)?;
                return Ok(config);
            }
        }
        
        Ok(Config::default())
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(proj_dirs) = ProjectDirs::from("", "", "interlinked") {
            let config_dir = proj_dirs.config_dir();
            fs::create_dir_all(config_dir)?;
            
            let config_path = config_dir.join("config.toml");
            let content = toml::to_string_pretty(self)?;
            fs::write(&config_path, content)?;
            
            println!("Config saved to: {}", config_path.display());
        }
        
        Ok(())
    }

    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}

pub struct FileIndexer {
    link_regex: Regex,
    tag_regex: Regex,
    config: Config,
}

impl FileIndexer {
    pub fn new(config: Config) -> Result<Self, regex::Error> {
        let link_regex = Regex::new(&config.link_pattern)?;
        let tag_regex = Regex::new(&config.tag_pattern)?;
        
        Ok(Self {
            link_regex,
            tag_regex,
            config,
        })
    }

    pub fn scan_directory<P: AsRef<Path>>(&self, dir: P) -> Result<FileIndex, Box<dyn std::error::Error>> {
        let mut index = FileIndex::new();
        
        for entry in WalkDir::new(dir)
            .into_iter()
            .filter_entry(|e| !self.is_ignored(e.path()))
        {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && self.is_text_file(path) {
                self.scan_file(path, &mut index)?;
            }
        }
        
        Ok(index)
    }

    fn is_ignored(&self, path: &Path) -> bool {
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            for pattern in &self.config.ignore_patterns {
                if pattern.starts_with('.') && name.starts_with(pattern) {
                    return true;
                } else if name == pattern {
                    return true;
                }
            }
        }
        false
    }

    fn is_text_file(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            let ext_lower = ext.to_lowercase();
            
            if self.config.scan_all_text {
                !self.config.binary_extensions.contains(&ext_lower)
            } else {
                self.config.file_extensions.contains(&ext_lower)
            }
        } else {
            if self.config.scan_all_text {
                self.is_likely_text_file(path)
            } else {
                false
            }
        }
    }

    fn is_likely_text_file(&self, path: &Path) -> bool {
        if let Ok(mut file) = std::fs::File::open(path) {
            use std::io::Read;
            let mut buffer = [0; 512];
            if let Ok(bytes_read) = file.read(&mut buffer) {
                if bytes_read == 0 {
                    return true;
                }
                
                let text = &buffer[..bytes_read];
                let null_count = text.iter().filter(|&&b| b == 0).count();
                let non_printable_count = text.iter()
                    .filter(|&&b| b < 32 && b != 9 && b != 10 && b != 13)
                    .count();
                
                let binary_threshold = 0.3;
                let binary_ratio = (null_count + non_printable_count) as f32 / bytes_read as f32;
                
                binary_ratio < binary_threshold
            } else {
                false
            }
        } else {
            false
        }
    }

    fn scan_file(&self, path: &Path, index: &mut FileIndex) -> Result<(), Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let file_path = path.to_string_lossy().to_string();
        
        for captures in self.link_regex.captures_iter(&content) {
            if let Some(link) = captures.get(1) {
                index.add_link(&file_path, link.as_str());
            }
        }
        
        for captures in self.tag_regex.captures_iter(&content) {
            if let Some(tags_line) = captures.get(1) {
                let tags: Vec<&str> = tags_line.as_str()
                    .split(',')
                    .map(|tag| tag.trim())
                    .filter(|tag| !tag.is_empty())
                    .collect();
                
                for tag in tags {
                    index.add_tag(&file_path, tag);
                }
            }
        }
        
        Ok(())
    }
}

#[derive(Parser)]
#[command(name = "interlinked")]
#[command(about = "A file indexer for Obsidian-style links and tags")]
pub struct Cli {
    #[arg(value_name = "DIRECTORY", help = "Directory to scan")]
    pub directory: Option<String>,
    
    #[arg(long, help = "Output as JSON")]
    pub json: bool,
    
    #[arg(long, value_name = "FILE", help = "Custom config file path")]
    pub config: Option<PathBuf>,
    
    #[arg(long, help = "Save default config and exit")]
    pub save_config: bool,
    
    #[arg(long, help = "Show config path and exit")]
    pub show_config_path: bool,
}

impl FileIndex {
    pub fn print_summary(&self, format: &OutputFormat) {
        match format {
            OutputFormat::Json => {
                if let Ok(json) = self.to_json() {
                    println!("{}", json);
                }
            },
            OutputFormat::Compact => {
                println!("Files: {}, Links: {}, Tags: {}", 
                    self.links.len(), self.backlinks.len(), self.tags.len());
            },
            OutputFormat::Pretty => {
                self.print_pretty();
            }
        }
    }
    
    pub fn print_pretty(&self) {
        println!("=== File Index Summary ===");
        println!("Total files with links: {}", self.links.len());
        println!("Total unique links: {}", self.backlinks.len());
        println!("Total unique tags: {}", self.tags.len());
        println!("Total files with tags: {}", self.file_tags.len());
        println!();
        
        if !self.tags.is_empty() {
            println!("=== Tags ===");
            for (tag, files) in &self.tags {
                println!("#{}: {} files", tag, files.len());
                for file in files {
                    println!("  - {}", file);
                }
            }
            println!();
        }
        
        if !self.links.is_empty() {
            println!("=== Links ===");
            for (file, links) in &self.links {
                println!("{}: {} links", file, links.len());
                for link in links {
                    println!("  -> [[{}]]", link);
                }
            }
            println!();
        }
        
        if !self.backlinks.is_empty() {
            println!("=== Backlinks ===");
            for (target, sources) in &self.backlinks {
                println!("[[{}]]: referenced by {} files", target, sources.len());
                for source in sources {
                    println!("  <- {}", source);
                }
            }
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    if cli.show_config_path {
        if let Some(proj_dirs) = ProjectDirs::from("", "", "interlinked") {
            println!("{}", proj_dirs.config_dir().join("config.toml").display());
        } else {
            println!("Could not determine config directory");
        }
        return Ok(());
    }
    
    let config = if let Some(config_path) = &cli.config {
        Config::load_from_path(config_path)?
    } else {
        Config::load()?
    };
    
    if cli.save_config {
        config.save()?;
        return Ok(());
    }
    
    let directory = cli.directory.as_deref().unwrap_or(".");
    println!("Scanning directory: {}", directory);
    
    let indexer = FileIndexer::new(config)?;
    let index = indexer.scan_directory(directory)?;
    
    let output_format = if cli.json {
        &OutputFormat::Json
    } else {
        &indexer.config.output_format
    };
    
    index.print_summary(output_format);
    
    Ok(())
}
