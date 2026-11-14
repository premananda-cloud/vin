//! Materialise a template tree on disk.

use crate::tree::Node;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy)]
pub struct Options {
    /// Overwrite files that already exist.
    pub force: bool,
    /// Put a `.gitkeep` in empty directories so git tracks them.
    pub gitkeep: bool,
    /// Print what would happen without touching the disk.
    pub dry_run: bool,
}

#[derive(Debug, Default)]
pub struct Report {
    pub dirs_created: usize,
    pub files_created: usize,
    pub files_overwritten: usize,
    pub files_skipped: Vec<PathBuf>,
}

/// Placeholder in template files that is replaced with the project name.
const TITLE_PLACEHOLDER: &str = "<Paper Title>";

pub fn materialize(
    base: &Path,
    children: &[(String, Node)],
    project_name: &str,
    opts: Options,
    report: &mut Report,
) -> Result<()> {
    ensure_dir(base, opts, report)?;
    write_children(base, children, project_name, opts, report)
}

fn ensure_dir(path: &Path, opts: Options, report: &mut Report) -> Result<()> {
    if path.is_dir() {
        return Ok(());
    }
    if !opts.dry_run {
        fs::create_dir_all(path).with_context(|| format!("creating {}", path.display()))?;
    }
    report.dirs_created += 1;
    Ok(())
}

fn write_children(
    dir: &Path,
    children: &[(String, Node)],
    project_name: &str,
    opts: Options,
    report: &mut Report,
) -> Result<()> {
    for (name, node) in children {
        let path = dir.join(name);
        match node {
            Node::Dir(sub) => {
                ensure_dir(&path, opts, report)?;
                if sub.is_empty() && opts.gitkeep {
                    write_file(&path.join(".gitkeep"), "", opts, report)?;
                } else {
                    write_children(&path, sub, project_name, opts, report)?;
                }
            }
            Node::File(content) => {
                let content = content.replace(TITLE_PLACEHOLDER, project_name);
                write_file(&path, &content, opts, report)?;
            }
        }
    }
    Ok(())
}

fn write_file(path: &Path, content: &str, opts: Options, report: &mut Report) -> Result<()> {
    let exists = path.exists();
    if exists && !opts.force {
        report.files_skipped.push(path.to_path_buf());
        return Ok(());
    }
    if !opts.dry_run {
        fs::write(path, content).with_context(|| format!("writing {}", path.display()))?;
    }
    if exists {
        report.files_overwritten += 1;
    } else {
        report.files_created += 1;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::parse;

    const OPTS: Options = Options {
        force: false,
        gitkeep: true,
        dry_run: false,
    };

    #[test]
    fn creates_tree_with_gitkeep_and_title() {
        let tree = parse("r:\n  data:\n    raw:\n  README.md: \"# <Paper Title>\"\n").unwrap();
        let tmp = tempfile::tempdir().unwrap();
        let base = tmp.path().join("proj");
        let mut rep = Report::default();
        materialize(&base, &tree.children, "My Paper", OPTS, &mut rep).unwrap();
        assert!(base.join("data/raw/.gitkeep").is_file());
        assert_eq!(
            fs::read_to_string(base.join("README.md")).unwrap(),
            "# My Paper"
        );
        assert_eq!(rep.files_created, 2);
    }

    #[test]
    fn does_not_overwrite_without_force() {
        let tree = parse("r:\n  a.txt: \"new\"\n").unwrap();
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("a.txt"), "old").unwrap();
        let mut rep = Report::default();
        materialize(tmp.path(), &tree.children, "p", OPTS, &mut rep).unwrap();
        assert_eq!(fs::read_to_string(tmp.path().join("a.txt")).unwrap(), "old");
        assert_eq!(rep.files_skipped.len(), 1);

        let forced = Options {
            force: true,
            ..OPTS
        };
        let mut rep = Report::default();
        materialize(tmp.path(), &tree.children, "p", forced, &mut rep).unwrap();
        assert_eq!(fs::read_to_string(tmp.path().join("a.txt")).unwrap(), "new");
        assert_eq!(rep.files_overwritten, 1);
    }

    #[test]
    fn dry_run_touches_nothing() {
        let tree = parse("r:\n  a:\n  f: \"x\"\n").unwrap();
        let tmp = tempfile::tempdir().unwrap();
        let base = tmp.path().join("proj");
        let opts = Options {
            dry_run: true,
            ..OPTS
        };
        let mut rep = Report::default();
        materialize(&base, &tree.children, "p", opts, &mut rep).unwrap();
        assert!(!base.exists());
        assert!(rep.dirs_created > 0);
    }
}
