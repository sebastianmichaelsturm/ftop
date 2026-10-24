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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ignored_directories() {
        assert!(should_ignore(Path::new(".git/HEAD"), false));
        assert!(should_ignore(Path::new("project/node_modules/pkg/index.js"), false));
        assert!(should_ignore(Path::new("target/debug/ftop.exe"), false));
        assert!(should_ignore(Path::new(".vscode/settings.json"), false));
        assert!(should_ignore(Path::new("__pycache__/mod.cpython-310.pyc"), false));
    }

    #[test]
    fn test_allowed_files() {
        assert!(!should_ignore(Path::new("src/main.rs"), false));
        assert!(!should_ignore(Path::new("Cargo.toml"), false));
        assert!(!should_ignore(Path::new("README.md"), false));
        assert!(!should_ignore(Path::new("docs/architecture.png"), false));
    }

    #[test]
    fn test_temporary_files() {
        assert!(should_ignore(Path::new("test.tmp"), false));
        assert!(should_ignore(Path::new("notes.txt.swp"), false));
        assert!(should_ignore(Path::new("file.txt~"), false));
        assert!(should_ignore(Path::new(".#save.txt"), false));
    }

    #[test]
    fn test_show_all_overrides() {
        assert!(!should_ignore(Path::new(".git/HEAD"), true));
        assert!(!should_ignore(Path::new("node_modules/react"), true));
        assert!(!should_ignore(Path::new("test.tmp"), true));
    }
}
