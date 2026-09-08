//! Read-only Git changes, source ranges, and reproducible run identities.

use super::source::{SourceInventory, SourceItem, fingerprint};
use serde::Serialize;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Debug, Clone, Serialize)]
pub struct ChangedFile {
    pub path: String,
    pub previous_path: Option<String>,
    pub status: String,
    pub ranges: Vec<ChangedRange>,
    pub items: Vec<SourceItem>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChangedRange {
    pub old_start: usize,
    pub old_lines: usize,
    pub new_start: usize,
    pub new_lines: usize,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ChangeSet {
    pub baseline: Option<String>,
    pub files: Vec<ChangedFile>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Provenance {
    pub analyzer_version: String,
    pub scoring_version: String,
    pub scope: String,
    pub revision: Option<String>,
    pub dirty: Option<bool>,
    pub source_fingerprint: String,
    pub config_fingerprint: Option<String>,
    pub effective_settings: crate::config::CouplingConfig,
    pub context_fingerprint: String,
    pub git_months: usize,
    pub git_used: bool,
    pub tests_excluded: bool,
    pub max_depth: Option<usize>,
    pub thresholds: BTreeMap<String, usize>,
}

pub fn git_root(path: &Path) -> Option<PathBuf> {
    let start = if path.is_file() { path.parent()? } else { path };
    let output = git(start, &["rev-parse", "--show-toplevel"]).ok()?;
    Some(PathBuf::from(output.trim()))
}

pub fn git(root: &Path, args: &[&str]) -> Result<String, std::io::Error> {
    let output = Command::new("git")
        .arg("--literal-pathspecs")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()?;
    if !output.status.success() {
        return Err(std::io::Error::other(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ));
    }
    String::from_utf8(output.stdout).map_err(std::io::Error::other)
}

pub fn resolve_ref(root: &Path, reference: &str) -> Result<String, std::io::Error> {
    let revision = format!("{reference}^{{commit}}");
    Ok(git(
        root,
        &["rev-parse", "--verify", "--end-of-options", &revision],
    )?
    .trim()
    .to_string())
}

pub fn read_changes(
    path: &Path,
    reference: &str,
    inventory: &SourceInventory,
    source_paths: &[PathBuf],
) -> Result<ChangeSet, std::io::Error> {
    let root = git_root(path)
        .ok_or_else(|| std::io::Error::other("--changed-since requires a Git repository"))?;
    let baseline = resolve_ref(&root, reference)?;
    let mut selected = fs::canonicalize(path)?;
    if selected
        .file_name()
        .is_some_and(|name| name == "Cargo.toml")
    {
        selected.pop();
    }
    let scope = selected
        .strip_prefix(&root)
        .map_err(std::io::Error::other)?
        .to_string_lossy();
    let scope = if scope.is_empty() { "." } else { &scope };
    let mut pathspecs = vec![scope.to_string()];
    for source in source_paths
        .iter()
        .filter(|source| !source.starts_with(&selected))
    {
        if let Ok(relative) = source.strip_prefix(&root) {
            pathspecs.push(relative.to_string_lossy().to_string());
        }
    }
    pathspecs.sort();
    pathspecs.dedup();
    let mut status_args = vec![
        "diff",
        "--no-ext-diff",
        "--no-textconv",
        "--name-status",
        "-z",
        "--find-renames",
        &baseline,
        "--",
    ];
    status_args.extend(pathspecs.iter().map(String::as_str));
    let status = git(&root, &status_args)?;
    let mut names = status.split('\0').filter(|part| !part.is_empty());
    let mut files = Vec::new();
    while let Some(code) = names.next() {
        let first = names
            .next()
            .ok_or_else(|| std::io::Error::other("invalid Git name-status output"))?;
        let renamed = code.starts_with('R') || code.starts_with('C');
        let filename = if renamed {
            names
                .next()
                .ok_or_else(|| std::io::Error::other("invalid Git rename output"))?
        } else {
            first
        };
        let previous = renamed.then(|| first.to_string());
        let patch = git(
            &root,
            &[
                "diff",
                "--no-ext-diff",
                "--no-textconv",
                "--no-color",
                "--unified=0",
                &baseline,
                "--",
                filename,
            ],
        )?;
        let ranges = parse_ranges(&patch);
        let absolute = root.join(filename).display().to_string();
        let mut items: Vec<_> = inventory
            .items
            .iter()
            .filter(|item| {
                item.file_path == absolute
                    && ranges
                        .iter()
                        .any(|range| overlaps(item, range.new_start, range.new_lines))
            })
            .cloned()
            .collect();
        if renamed && ranges.is_empty() {
            items = inventory
                .items
                .iter()
                .filter(|item| item.file_path == absolute)
                .cloned()
                .collect();
        }
        if code.starts_with('D') || ranges.iter().any(|range| range.old_lines > 0) {
            let blob = git(&root, &["show", &format!("{baseline}:{first}")])?;
            let module = inventory
                .items
                .iter()
                .find(|item| item.file_path == absolute)
                .map(|item| item.module.clone())
                .unwrap_or_else(|| {
                    Path::new(first)
                        .file_stem()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string()
                });
            let old_items =
                SourceInventory::from_source(&blob, &root.join(first), &module, false).items;
            for mut item in old_items {
                if (code.starts_with('D')
                    || ranges
                        .iter()
                        .any(|range| overlaps(&item, range.old_start, range.old_lines)))
                    && !items
                        .iter()
                        .any(|current| current.module == item.module && current.name == item.name)
                {
                    item.revision = Some(baseline.clone());
                    items.push(item);
                }
            }
        }
        files.push(ChangedFile {
            path: filename.into(),
            previous_path: previous,
            status: code.into(),
            ranges,
            items,
        });
    }
    let mut untracked_args = vec!["ls-files", "--others", "--exclude-standard", "-z", "--"];
    untracked_args.extend(pathspecs.iter().map(String::as_str));
    let untracked = git(&root, &untracked_args)?;
    for filename in untracked.split('\0').filter(|part| !part.is_empty()) {
        let absolute = root.join(filename).display().to_string();
        let items = inventory
            .items
            .iter()
            .filter(|item| item.file_path == absolute)
            .cloned()
            .collect();
        files.push(ChangedFile {
            path: filename.into(),
            previous_path: None,
            status: "?".into(),
            ranges: vec![],
            items,
        });
    }
    files.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(ChangeSet { baseline: Some(baseline), files, notes: vec!["Changes include staged, unstaged and untracked paths; item ranges come from Rust source spans. Deleted items need baseline dependencies for complete impact.".into()] })
}

fn overlaps(item: &SourceItem, start: usize, length: usize) -> bool {
    if length == 0 {
        return false;
    }
    let end = start.saturating_add(length.saturating_sub(1));
    item.line <= end && item.end_line >= start
}

fn parse_ranges(patch: &str) -> Vec<ChangedRange> {
    patch
        .lines()
        .filter_map(|line| {
            if !line.starts_with("@@ ") {
                return None;
            }
            let mut fields = line.split_whitespace().skip(1);
            let (old_start, old_lines) = parse_range(fields.next()?.strip_prefix('-')?)?;
            let (new_start, new_lines) = parse_range(fields.next()?.strip_prefix('+')?)?;
            Some(ChangedRange {
                old_start,
                old_lines,
                new_start,
                new_lines,
            })
        })
        .collect()
}

fn parse_range(range: &str) -> Option<(usize, usize)> {
    match range.split_once(',') {
        Some((start, count)) => Some((start.parse().ok()?, count.parse().ok()?)),
        None => Some((range.parse().ok()?, 1)),
    }
}

pub fn source_fingerprint(inventory: &SourceInventory, root: &Path) -> String {
    let source: String = inventory
        .fingerprints
        .iter()
        .map(|(path, hash)| {
            let relative = Path::new(path)
                .strip_prefix(root)
                .unwrap_or(Path::new(path));
            format!("{}\0{hash}\n", relative.display())
        })
        .collect();
    fingerprint(source.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn deleted_items_in_surviving_file_are_included_with_manifest_scope() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname='fixture'\nversion='0.1.0'\n",
        )
        .unwrap();
        let file = dir.path().join("lib.rs");
        fs::write(&file, "pub fn removed() {}\npub fn retained() {}\n").unwrap();
        git(dir.path(), &["init", "-q"]).unwrap();
        git(dir.path(), &["add", "."]).unwrap();
        git(
            dir.path(),
            &[
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "commit",
                "-qm",
                "baseline",
            ],
        )
        .unwrap();
        let source = "pub fn retained() {}\n";
        fs::write(&file, source).unwrap();
        let inventory = SourceInventory::from_source(source, &file, "lib", false);
        let changes =
            read_changes(&dir.path().join("Cargo.toml"), "HEAD", &inventory, &[]).unwrap();
        assert!(
            changes
                .files
                .iter()
                .flat_map(|file| &file.items)
                .any(|item| item.name == "removed"
                    && item.revision.as_deref() == changes.baseline.as_deref())
        );
    }
    #[test]
    fn parses_additions_and_deletions_without_count_defaults() {
        let ranges = parse_ranges("@@ -2,0 +3,4 @@\n+x\n@@ -8 +12,0 @@\n-y\n");
        assert_eq!(ranges.len(), 2);
        assert_eq!(ranges[0].old_lines, 0);
        assert_eq!(ranges[1].old_lines, 1);
        assert_eq!(ranges[1].new_lines, 0);
    }
}
