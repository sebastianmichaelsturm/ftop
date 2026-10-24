use std::path::Path;

const IGNORED_COMPONENTS: &[&str] = &[
    ".git",
    "node_modules",
    "target",
    ".idea",
    ".vscode",
    "__pycache__",
    ".next",
    ".cache",
    ".gemini",
];

/// Determines whether a given relative path should be ignored by default.
pub fn should_ignore(path: &Path, show_all: bool) -> bool {
    if show_all {
        return false;
    }

    for component in path.components() {
        if let Some(comp_str) = component.as_os_str().to_str() {
            // Ignore standard high-churn or metadata directories
            if IGNORED_COMPONENTS.contains(&comp_str) {
                return true;
            }
            // Ignore temporary files (e.g. swap files, editor temp files)
            if comp_str.ends_with(".tmp")
                || comp_str.ends_with(".swp")
                || comp_str.ends_with("~")
                || comp_str.starts_with(".#")
            {
                return true;
            }
        }
    }

    false
}
