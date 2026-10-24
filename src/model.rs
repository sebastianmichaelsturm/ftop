use chrono::{DateTime, Local};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeType {
    Created,
    Modified,
    Deleted,
    Renamed,
    Other,
}

impl ChangeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ChangeType::Created => "CREATED",
            ChangeType::Modified => "MODIFIED",
            ChangeType::Deleted => "DELETED",
            ChangeType::Renamed => "RENAMED",
            ChangeType::Other => "OTHER",
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileRecord {
    pub display_path: String,
    pub absolute_path: PathBuf,
    pub event_type: ChangeType,
    pub change_count: u64,
    pub size_bytes: Option<u64>,
    pub last_updated: DateTime<Local>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortMode {
    Recent,
    ChangeCount,
    Size,
    Path,
}

impl SortMode {
    pub fn next(&self) -> Self {
        match self {
            SortMode::Recent => SortMode::ChangeCount,
            SortMode::ChangeCount => SortMode::Size,
            SortMode::Size => SortMode::Path,
            SortMode::Path => SortMode::Recent,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            SortMode::Recent => "Most Recent",
            SortMode::ChangeCount => "Highest Changes",
            SortMode::Size => "File Size",
            SortMode::Path => "Path Alphabetical",
        }
    }
}

pub fn format_size(bytes: Option<u64>) -> String {
    match bytes {
        Some(b) => {
            const KB: f64 = 1024.0;
            const MB: f64 = KB * 1024.0;
            const GB: f64 = MB * 1024.0;
            let bf = b as f64;
            if bf < KB {
                format!("{} B", b)
            } else if bf < MB {
                format!("{:.1} KB", bf / KB)
            } else if bf < GB {
                format!("{:.1} MB", bf / MB)
            } else {
                format!("{:.2} GB", bf / GB)
            }
        }
        None => "-".to_string(),
    }
}
