use std::path::Path;

/// Working directory for tools that also accept a file as the analysis scope.
pub(crate) fn containing_directory(path: &Path) -> &Path {
    if path.is_file() {
        path.parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."))
    } else {
        path
    }
}
