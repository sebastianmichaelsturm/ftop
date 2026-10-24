use crate::filter::should_ignore;
use crate::model::{ChangeType, FileRecord, SortMode};
use chrono::Local;
use notify::event::ModifyKind;
use notify::{Event, EventKind};
use ratatui::widgets::TableState;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
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
                absolute_path: path,
                event_type: change_type,
                change_count: 0,
                size_bytes: size,
                last_updated: now,
            });

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

    pub fn get_selected_file(&self) -> Option<PathBuf> {
        let list = self.sorted_records();
        let idx = self.table_state.selected()?;
        list.get(idx).map(|rec| rec.absolute_path.clone())
    }

    pub fn open_selected(&self) {
        if let Some(path) = self.get_selected_file() {
            if path.exists() {
                #[cfg(target_os = "windows")]
                {
                    let _ = std::process::Command::new("cmd")
                        .args(["/C", "start", "", &path.to_string_lossy()])
                        .spawn();
                }
                #[cfg(target_os = "macos")]
                {
                    let _ = std::process::Command::new("open").arg(path).spawn();
                }
                #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
                {
                    if let Ok(editor) = std::env::var("EDITOR") {
                        let _ = std::process::Command::new(editor).arg(path).spawn();
                    } else {
                        let _ = std::process::Command::new("xdg-open").arg(path).spawn();
                    }
                }
            }
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
