use crate::filter::should_ignore;
use crate::model::{ChangeType, DiffLine, DiffLineKind, FileRecord, SortMode, ViewerState};
use chrono::Local;
use notify::event::ModifyKind;
use notify::{Event, EventKind};
use ratatui::widgets::TableState;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

pub struct App {
    pub root_path: PathBuf,
    pub records: HashMap<PathBuf, FileRecord>,
    pub table_state: TableState,
    pub sort_mode: SortMode,
    pub show_all: bool,
    pub total_events: u64,
    pub start_time: Instant,
    pub is_paused: bool,
    pub frozen_records: Option<Vec<FileRecord>>,
    pub filter_query: String,
    pub is_filtering: bool,
    pub viewer: Option<ViewerState>,
}

impl App {
    pub fn new(root_path: PathBuf, show_all: bool) -> Self {
        let mut table_state = TableState::default();
        table_state.select(Some(0));

        Self {
            root_path,
            records: HashMap::new(),
            table_state,
            sort_mode: SortMode::Recent,
            show_all,
            total_events: 0,
            start_time: Instant::now(),
            is_paused: false,
            frozen_records: None,
            filter_query: String::new(),
            is_filtering: false,
            viewer: None,
        }
    }

    pub fn process_event(&mut self, event: Event) {
        // Skip pure access/read events to focus on file mutations
        if matches!(event.kind, EventKind::Access(_)) {
            return;
        }

        let change_type = match event.kind {
            EventKind::Create(_) => ChangeType::Created,
            EventKind::Modify(ModifyKind::Name(_)) => ChangeType::Renamed,
            EventKind::Modify(_) => ChangeType::Modified,
            EventKind::Remove(_) => ChangeType::Deleted,
            _ => ChangeType::Other,
        };

        for path in event.paths {
            let rel_path = match path.strip_prefix(&self.root_path) {
                Ok(p) => p,
                Err(_) => path.as_path(),
            };

            if should_ignore(rel_path, self.show_all) {
                continue;
            }

            self.total_events += 1;

            let display_str = rel_path.to_string_lossy().to_string();
            let size = if change_type != ChangeType::Deleted && path.exists() {
                fs::metadata(&path).ok().map(|m| m.len())
            } else {
                None
            };

            let now = Local::now();
            let entry = self.records.entry(path.clone()).or_insert_with(|| FileRecord {
                display_path: if display_str.is_empty() { ".".to_string() } else { display_str },
                absolute_path: path.clone(),
                event_type: change_type,
                change_count: 0,
                size_bytes: size,
                last_updated: now,
                previous_content: None,
            });

            // Capture initial baseline if not captured yet
            if entry.previous_content.is_none() && path.is_file() {
                if let Ok(meta) = fs::metadata(&path) {
                    if meta.len() < 500 * 1024 && !is_binary_file(&path) {
                        entry.previous_content = fs::read_to_string(&path).ok();
                    }
                }
            }

            entry.event_type = change_type;
            entry.change_count += 1;
            entry.size_bytes = size;
            entry.last_updated = now;
        }
    }

    pub fn toggle_pause(&mut self) {
        self.is_paused = !self.is_paused;
        if self.is_paused {
            self.frozen_records = Some(self.sorted_records_internal().into_iter().cloned().collect());
        } else {
            self.frozen_records = None;
        }
    }

    pub fn sorted_records(&self) -> Vec<&FileRecord> {
        if self.is_paused {
            if let Some(ref frozen) = self.frozen_records {
                return frozen.iter().collect();
            }
        }
        self.sorted_records_internal()
    }

    fn sorted_records_internal(&self) -> Vec<&FileRecord> {
        let query = self.filter_query.to_lowercase();
        let mut list: Vec<&FileRecord> = self
            .records
            .values()
            .filter(|r| query.is_empty() || r.display_path.to_lowercase().contains(&query))
            .collect();

        match self.sort_mode {
            SortMode::Recent => {
                list.sort_by(|a, b| b.last_updated.cmp(&a.last_updated));
            }
            SortMode::ChangeCount => {
                list.sort_by(|a, b| b.change_count.cmp(&a.change_count));
            }
            SortMode::Size => {
                list.sort_by(|a, b| b.size_bytes.unwrap_or(0).cmp(&a.size_bytes.unwrap_or(0)));
            }
            SortMode::Path => {
                list.sort_by(|a, b| a.display_path.cmp(&b.display_path));
            }
        }
        list
    }

    pub fn cycle_sort(&mut self) {
        self.sort_mode = self.sort_mode.next();
        if self.is_paused {
            // Re-sort frozen view
            self.frozen_records = Some(self.sorted_records_internal().into_iter().cloned().collect());
        }
    }

    pub fn clear(&mut self) {
        self.records.clear();
        self.frozen_records = None;
        self.total_events = 0;
        self.table_state.select(Some(0));
    }

    pub fn get_selected_file_record(&self) -> Option<&FileRecord> {
        let list = self.sorted_records();
        let idx = self.table_state.selected()?;
        list.get(idx).copied()
    }

    pub fn get_selected_file(&self) -> Option<PathBuf> {
        self.get_selected_file_record().map(|r| r.absolute_path.clone())
    }

    pub fn open_viewer(&mut self) {
        let (abs_path, display_path) = match self.get_selected_file_record() {
            Some(rec) => (rec.absolute_path.clone(), rec.display_path.clone()),
            None => return,
        };

        if !abs_path.exists() {
            let lines = vec![
                DiffLine {
                    text: format!("File does not exist on disk: {}", display_path),
                    kind: DiffLineKind::Deletion,
                },
                DiffLine {
                    text: "The file was either deleted or moved.".to_string(),
                    kind: DiffLineKind::Info,
                },
            ];
            self.viewer = Some(ViewerState::new(
                abs_path,
                display_path,
                "File Deleted".to_string(),
                lines.clone(),
                lines,
            ));
            return;
        }

        if is_binary_file(&abs_path) {
            let size = fs::metadata(&abs_path).map(|m| m.len()).unwrap_or(0);
            let lines = vec![
                DiffLine {
                    text: format!("Binary file: {}", display_path),
                    kind: DiffLineKind::Header,
                },
                DiffLine {
                    text: format!("Size: {} ({} bytes)", crate::model::format_size(Some(size)), size),
                    kind: DiffLineKind::Info,
                },
                DiffLine {
                    text: "Binary files cannot be rendered as text diff in terminal.".to_string(),
                    kind: DiffLineKind::Info,
                },
            ];
            self.viewer = Some(ViewerState::new(
                abs_path,
                display_path,
                "Binary File".to_string(),
                lines.clone(),
                lines,
            ));
            return;
        }

        // 1. Build full_lines (entire file content with line numbers)
        let full_lines: Vec<DiffLine> = match fs::read_to_string(&abs_path) {
            Ok(content) => content
                .lines()
                .enumerate()
                .map(|(i, l)| DiffLine {
                    text: format!("{:4} │ {}", i + 1, l),
                    kind: DiffLineKind::Context,
                })
                .collect(),
            Err(_) => vec![DiffLine {
                text: format!("Failed to read file: {}", display_path),
                kind: DiffLineKind::Deletion,
            }],
        };

        // 2. Build diff_only_lines
        let mut diff_only_lines = Vec::new();

        // 2a. Try Git diff against HEAD
        if let Some(git_lines) = get_git_diff(&self.root_path, &abs_path) {
            for line in git_lines {
                if line.kind == DiffLineKind::Addition
                    || line.kind == DiffLineKind::Deletion
                    || line.kind == DiffLineKind::Header
                {
                    diff_only_lines.push(line);
                }
            }
        }

        // 2b. If no git diff, try in-memory session diff
        if diff_only_lines.is_empty() {
            let prev_content = self.records.get(&abs_path).and_then(|r| r.previous_content.as_ref());
            if let Some(old) = prev_content {
                if let Ok(new) = fs::read_to_string(&abs_path) {
                    if old != &new {
                        let diff = similar::TextDiff::from_lines(old, &new);
                        diff_only_lines.push(DiffLine {
                            text: format!("@@ Session Diff: {} @@", display_path),
                            kind: DiffLineKind::Header,
                        });
                        for change in diff.iter_all_changes() {
                            let kind = match change.tag() {
                                similar::ChangeTag::Delete => DiffLineKind::Deletion,
                                similar::ChangeTag::Insert => DiffLineKind::Addition,
                                similar::ChangeTag::Equal => DiffLineKind::Context,
                            };
                            let prefix = match change.tag() {
                                similar::ChangeTag::Delete => "-",
                                similar::ChangeTag::Insert => "+",
                                similar::ChangeTag::Equal => " ",
                            };
                            if kind != DiffLineKind::Context {
                                let line_val = change.value().trim_end_matches(['\r', '\n']);
                                diff_only_lines.push(DiffLine {
                                    text: format!("{}{}", prefix, line_val),
                                    kind,
                                });
                            }
                        }
                    }
                }
            }
        }

        // 2c. If still no diff, add clear informative notice
        if diff_only_lines.is_empty() {
            diff_only_lines.push(DiffLine {
                text: "[No line modifications detected in this file. Press Tab to view full file.]"
                    .to_string(),
                kind: DiffLineKind::Info,
            });
        }

        let title = format!("Diff Viewer ─ {}", display_path);
        self.viewer = Some(ViewerState::new(
            abs_path,
            display_path,
            title,
            full_lines,
            diff_only_lines,
        ));
    }

    pub fn close_viewer(&mut self) {
        self.viewer = None;
    }

    pub fn toggle_viewer_diff_mode(&mut self) {
        if let Some(ref mut viewer) = self.viewer {
            viewer.toggle_diff_mode();
        }
    }

    pub fn scroll_viewer_up(&mut self, amount: usize) {
        if let Some(ref mut viewer) = self.viewer {
            viewer.scroll_up(amount);
        }
    }

    pub fn scroll_viewer_down(&mut self, amount: usize, visible_height: usize) {
        if let Some(ref mut viewer) = self.viewer {
            viewer.scroll_down(amount, visible_height);
        }
    }

    pub fn scroll_viewer_top(&mut self) {
        if let Some(ref mut viewer) = self.viewer {
            viewer.scroll_top();
        }
    }

    pub fn scroll_viewer_bottom(&mut self, visible_height: usize) {
        if let Some(ref mut viewer) = self.viewer {
            viewer.scroll_bottom(visible_height);
        }
    }

    pub fn next_row(&mut self) {
        let count = self.sorted_records().len();
        if count == 0 {
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => {
                if i >= count - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    pub fn previous_row(&mut self) {
        let count = self.sorted_records().len();
        if count == 0 {
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    count - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }
}

fn is_binary_file(path: &Path) -> bool {
    // Check known binary extensions first
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        let ext_lower = ext.to_lowercase();
        if matches!(
            ext_lower.as_str(),
            "exe" | "dll" | "so" | "dylib" | "bin" | "png" | "jpg" | "jpeg" | "gif" | "ico"
                | "pdf" | "zip" | "tar" | "gz" | "7z" | "woff" | "woff2" | "ttf" | "pdb"
        ) {
            return true;
        }
    }
    // Inspect first 1024 bytes for null bytes
    if let Ok(mut file) = fs::File::open(path) {
        use std::io::Read;
        let mut buf = [0u8; 1024];
        if let Ok(n) = file.read(&mut buf) {
            return buf[..n].contains(&0);
        }
    }
    false
}

fn get_git_diff(repo_root: &Path, file_path: &Path) -> Option<Vec<DiffLine>> {
    let output = std::process::Command::new("git")
        .current_dir(repo_root)
        .args(["diff", "HEAD", "--"])
        .arg(file_path)
        .output()
        .ok()?;

    if !output.status.success() || output.stdout.is_empty() {
        return None;
    }

    let diff_text = String::from_utf8_lossy(&output.stdout);
    if diff_text.trim().is_empty() {
        return None;
    }

    let mut lines = Vec::new();
    for raw_line in diff_text.lines() {
        let kind = if raw_line.starts_with('+') && !raw_line.starts_with("+++") {
            DiffLineKind::Addition
        } else if raw_line.starts_with('-') && !raw_line.starts_with("---") {
            DiffLineKind::Deletion
        } else if raw_line.starts_with("@@")
            || raw_line.starts_with("diff ")
            || raw_line.starts_with("index ")
        {
            DiffLineKind::Header
        } else {
            DiffLineKind::Context
        };
        lines.push(DiffLine {
            text: raw_line.to_string(),
            kind,
        });
    }

    if lines.is_empty() {
        None
    } else {
        Some(lines)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use notify::event::CreateKind;

    #[test]
    fn test_app_initial_state() {
        let app = App::new(PathBuf::from("/test"), false);
        assert_eq!(app.records.len(), 0);
        assert_eq!(app.total_events, 0);
        assert_eq!(app.sort_mode, SortMode::Recent);
        assert!(!app.is_paused);
        assert!(!app.is_filtering);
        assert_eq!(app.filter_query, "");
        assert!(app.viewer.is_none());
    }

    #[test]
    fn test_process_event_adds_record() {
        let mut app = App::new(PathBuf::from("."), false);
        let mut event = Event::new(EventKind::Create(CreateKind::File));
        event.paths.push(PathBuf::from("./test_file.rs"));

        app.process_event(event);

        assert_eq!(app.total_events, 1);
        assert_eq!(app.records.len(), 1);
        let rec = app.records.values().next().unwrap();
        assert_eq!(rec.event_type, ChangeType::Created);
        assert_eq!(rec.change_count, 1);
    }

    #[test]
    fn test_filter_query_filters_records() {
        let mut app = App::new(PathBuf::from("."), false);
        let mut ev1 = Event::new(EventKind::Create(CreateKind::File));
        ev1.paths.push(PathBuf::from("./src/main.rs"));
        app.process_event(ev1);

        let mut ev2 = Event::new(EventKind::Create(CreateKind::File));
        ev2.paths.push(PathBuf::from("./README.md"));
        app.process_event(ev2);

        assert_eq!(app.sorted_records().len(), 2);

        app.filter_query = "main".to_string();
        let filtered = app.sorted_records();
        assert_eq!(filtered.len(), 1);
        assert!(filtered[0].display_path.contains("main.rs"));

        app.filter_query = "xyz".to_string();
        assert_eq!(app.sorted_records().len(), 0);
    }

    #[test]
    fn test_pause_freezes_snapshot() {
        let mut app = App::new(PathBuf::from("."), false);
        let mut ev1 = Event::new(EventKind::Create(CreateKind::File));
        ev1.paths.push(PathBuf::from("./first.rs"));
        app.process_event(ev1);

        app.toggle_pause();
        assert!(app.is_paused);
        assert_eq!(app.sorted_records().len(), 1);

        // Add another event while paused
        let mut ev2 = Event::new(EventKind::Create(CreateKind::File));
        ev2.paths.push(PathBuf::from("./second.rs"));
        app.process_event(ev2);

        // Active records has 2, but sorted_records (frozen display) still has 1
        assert_eq!(app.records.len(), 2);
        assert_eq!(app.sorted_records().len(), 1);

        app.toggle_pause();
        assert!(!app.is_paused);
        assert_eq!(app.sorted_records().len(), 2);
    }

    #[test]
    fn test_sorting_modes() {
        let mut app = App::new(PathBuf::from("."), false);

        let mut ev1 = Event::new(EventKind::Create(CreateKind::File));
        ev1.paths.push(PathBuf::from("./b_file.rs"));
        app.process_event(ev1);

        let mut ev2 = Event::new(EventKind::Create(CreateKind::File));
        ev2.paths.push(PathBuf::from("./a_file.rs"));
        app.process_event(ev2);

        // Trigger another event on a_file to give it higher change count
        let mut ev3 = Event::new(EventKind::Modify(ModifyKind::Any));
        ev3.paths.push(PathBuf::from("./a_file.rs"));
        app.process_event(ev3);

        // Sort by Path
        app.sort_mode = SortMode::Path;
        let sorted_path = app.sorted_records();
        assert!(sorted_path[0].display_path < sorted_path[1].display_path);

        // Sort by ChangeCount
        app.sort_mode = SortMode::ChangeCount;
        let sorted_count = app.sorted_records();
        assert!(sorted_count[0].change_count >= sorted_count[1].change_count);
    }

    #[test]
    fn test_binary_detection() {
        let exe_path = PathBuf::from("test.exe");
        assert!(is_binary_file(&exe_path));

        let rs_path = PathBuf::from("test.rs");
        // Non-existent or empty text file
        assert!(!is_binary_file(&rs_path));
    }
}
