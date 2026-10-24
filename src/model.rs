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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(None), "-");
        assert_eq!(format_size(Some(0)), "0 B");
        assert_eq!(format_size(Some(512)), "512 B");
        assert_eq!(format_size(Some(1024)), "1.0 KB");
        assert_eq!(format_size(Some(1536)), "1.5 KB");
        assert_eq!(format_size(Some(1024 * 1024)), "1.0 MB");
        assert_eq!(format_size(Some(1024 * 1024 * 1024)), "1.00 GB");
    }

    #[test]
    fn test_sort_mode_cycle() {
        let mut mode = SortMode::Recent;
        mode = mode.next();
        assert_eq!(mode, SortMode::ChangeCount);
        mode = mode.next();
        assert_eq!(mode, SortMode::Size);
        mode = mode.next();
        assert_eq!(mode, SortMode::Path);
        mode = mode.next();
        assert_eq!(mode, SortMode::Recent);
    }

    #[test]
    fn test_change_type_as_str() {
        assert_eq!(ChangeType::Created.as_str(), "CREATED");
        assert_eq!(ChangeType::Modified.as_str(), "MODIFIED");
        assert_eq!(ChangeType::Deleted.as_str(), "DELETED");
        assert_eq!(ChangeType::Renamed.as_str(), "RENAMED");
        assert_eq!(ChangeType::Other.as_str(), "OTHER");
    }
}
