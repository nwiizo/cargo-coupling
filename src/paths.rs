use std::path::{Component, Path, PathBuf};

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

/// Compare paths lexically, preserving symlink names used in glob patterns.
pub(crate) fn absolute_normalized_path(path: &Path) -> std::io::Result<PathBuf> {
    let mut result = PathBuf::new();
    for component in std::path::absolute(path)?.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                result.pop();
            }
            other => result.push(other.as_os_str()),
        }
    }
    Ok(result)
}

/// Find the nearest named file, checking names in priority order in each directory.
pub(crate) fn find_in_ancestors(start: &Path, names: &[&str]) -> std::io::Result<Option<PathBuf>> {
    let absolute = absolute_normalized_path(start)?;
    for dir in containing_directory(&absolute).ancestors() {
        for name in names {
            let path = dir.join(name);
            if path.try_exists()? {
                return Ok(Some(path));
            }
        }
    }
    Ok(None)
}

pub(crate) fn normalized_path(path: &Path) -> PathBuf {
    absolute_normalized_path(path).unwrap_or_else(|_| path.to_owned())
}

pub(crate) fn config_matching_path(file: &Path, root: Option<&Path>) -> String {
    let file = normalized_path(file);
    let relative = root
        .map(normalized_path)
        .and_then(|root| file.strip_prefix(root).ok());
    relative
        .unwrap_or(&file)
        .to_string_lossy()
        .replace('\\', "/")
}
