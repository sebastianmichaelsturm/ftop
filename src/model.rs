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
    pub previous_content: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffLineKind {
    Addition, // Green (+)
    Deletion, // Red (-)
    Header,   // Cyan (@@, diff headers)
    Context,  // Normal text
    Info,     // Yellow / Gray notice
}

#[derive(Debug, Clone)]
pub struct DiffLine {
    pub text: String,
    pub kind: DiffLineKind,
}

#[derive(Debug, Clone)]
pub struct ViewerState {
    pub path: PathBuf,
    pub display_path: String,
    pub title: String,
    pub lines: Vec<DiffLine>,
    pub scroll_offset: usize,
}

impl ViewerState {
    pub fn new(path: PathBuf, display_path: String, title: String, lines: Vec<DiffLine>) -> Self {
        Self {
            path,
            display_path,
            title,
            lines,
            scroll_offset: 0,
        }
    }

    pub fn total_lines(&self) -> usize {
        self.lines.len()
    }

    pub fn scroll_up(&mut self, amount: usize) {
        if self.scroll_offset >= amount {
            self.scroll_offset -= amount;
        } else {
            self.scroll_offset = 0;
        }
    }

    pub fn scroll_down(&mut self, amount: usize, visible_height: usize) {
        let max_offset = if self.lines.len() > visible_height {
            self.lines.len() - visible_height
        } else {
            0
        };
        self.scroll_offset = (self.scroll_offset + amount).min(max_offset);
    }

    pub fn scroll_top(&mut self) {
        self.scroll_offset = 0;
    }

    pub fn scroll_bottom(&mut self, visible_height: usize) {
        if self.lines.len() > visible_height {
            self.scroll_offset = self.lines.len() - visible_height;
        } else {
            self.scroll_offset = 0;
        }
    }
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

    #[test]
    fn test_viewer_state_scrolling() {
        let lines = (0..50)
            .map(|i| DiffLine {
                text: format!("line {}", i),
                kind: DiffLineKind::Context,
            })
            .collect();
        let mut viewer = ViewerState::new(
            PathBuf::from("test.txt"),
            "test.txt".to_string(),
            "Test".to_string(),
            lines,
        );

        assert_eq!(viewer.total_lines(), 50);
        assert_eq!(viewer.scroll_offset, 0);

        // Scroll down
        viewer.scroll_down(5, 20);
        assert_eq!(viewer.scroll_offset, 5);

        // Scroll up
        viewer.scroll_up(2);
        assert_eq!(viewer.scroll_offset, 3);

        // Scroll up past 0 bounds
        viewer.scroll_up(10);
        assert_eq!(viewer.scroll_offset, 0);

        // Scroll to bottom (50 lines, 20 visible -> max offset 30)
        viewer.scroll_bottom(20);
        assert_eq!(viewer.scroll_offset, 30);

        // Scroll to top
        viewer.scroll_top();
        assert_eq!(viewer.scroll_offset, 0);
    }
}
