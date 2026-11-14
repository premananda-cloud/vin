//! In-memory model of a template: a tree of directories and files, parsed from YAML.
//!
//! YAML convention:
//!   mapping -> directory (recurse) | null -> empty directory | string -> file content

use anyhow::{anyhow, bail, Result};
use serde_yaml::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Dir(Vec<(String, Node)>),
    File(String),
}

#[derive(Debug, Clone)]
pub struct Tree {
    /// The single root key of the YAML file (e.g. `paper_template_for_ai`).
    pub root_name: String,
    pub children: Vec<(String, Node)>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Stats {
    pub dirs: usize,
    pub files: usize,
}

pub fn parse(yaml: &str) -> Result<Tree> {
    let value: Value = serde_yaml::from_str(yaml)?;
    let Value::Mapping(root) = value else {
        bail!("template must be a YAML mapping with a single root key");
    };
    if root.len() != 1 {
        bail!(
            "template must have exactly one root key, found {}",
            root.len()
        );
    }
    let (key, val) = root.into_iter().next().unwrap();
    let root_name = key_name(&key)?;
    let children = match to_node(&root_name, val)? {
        Node::Dir(c) => c,
        Node::File(_) => bail!("root key `{root_name}` must be a directory, not a file"),
    };
    Ok(Tree {
        root_name,
        children,
    })
}

fn key_name(key: &Value) -> Result<String> {
    let name = match key {
        Value::String(s) => s.clone(),
        other => bail!("directory/file names must be strings, found {other:?}"),
    };
    validate_name(&name)?;
    Ok(name)
}

/// Reject names that could escape the target directory or are otherwise invalid.
pub fn validate_name(name: &str) -> Result<()> {
    if name.is_empty() || name == "." || name == ".." {
        bail!("invalid name `{name}`");
    }
    if name.contains('/') || name.contains('\\') || name.contains('\0') {
        bail!("invalid name `{name}`: must not contain path separators");
    }
    Ok(())
}

fn to_node(path: &str, value: Value) -> Result<Node> {
    match value {
        Value::Null => Ok(Node::Dir(Vec::new())),
        Value::String(s) => Ok(Node::File(s)),
        Value::Mapping(map) => {
            let mut children = Vec::with_capacity(map.len());
            let mut seen = std::collections::HashSet::new();
            for (k, v) in map {
                let name = key_name(&k)?;
                if !seen.insert(name.clone()) {
                    bail!("duplicate entry `{name}` in `{path}`");
                }
                let child = to_node(&format!("{path}/{name}"), v)?;
                children.push((name, child));
            }
            Ok(Node::Dir(children))
        }
        other => Err(anyhow!(
            "`{path}`: expected a mapping (directory), null (empty directory) or string (file), found {other:?}"
        )),
    }
}

impl Tree {
    /// Counts directories (excluding the root) and files.
    pub fn stats(&self) -> Stats {
        let mut s = Stats::default();
        count(&self.children, &mut s);
        s
    }

    /// Render as an ASCII tree. `max_depth` limits nesting (1 = top level only).
    pub fn render(&self, name: &str, max_depth: Option<usize>) -> String {
        let mut out = format!("{name}/\n");
        render_children(&self.children, "", 1, max_depth, &mut out);
        out
    }
}

fn count(children: &[(String, Node)], s: &mut Stats) {
    for (_, node) in children {
        match node {
            Node::Dir(c) => {
                s.dirs += 1;
                count(c, s);
            }
            Node::File(_) => s.files += 1,
        }
    }
}

fn render_children(
    children: &[(String, Node)],
    prefix: &str,
    depth: usize,
    max_depth: Option<usize>,
    out: &mut String,
) {
    let last = children.len().saturating_sub(1);
    for (i, (name, node)) in children.iter().enumerate() {
        let (branch, next_prefix) = if i == last {
            ("└── ", format!("{prefix}    "))
        } else {
            ("├── ", format!("{prefix}│   "))
        };
        match node {
            Node::Dir(c) => {
                out.push_str(&format!("{prefix}{branch}{name}/\n"));
                if max_depth.is_none_or(|m| depth < m) {
                    render_children(c, &next_prefix, depth + 1, max_depth, out);
                } else if !c.is_empty() {
                    out.push_str(&format!("{next_prefix}└── …\n"));
                }
            }
            Node::File(_) => out.push_str(&format!("{prefix}{branch}{name}\n")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_dirs_and_files() {
        let t = parse("root:\n  a:\n  b:\n    c:\n  f.txt: \"hi\"\n").unwrap();
        assert_eq!(t.root_name, "root");
        assert_eq!(t.stats(), Stats { dirs: 3, files: 1 });
    }

    #[test]
    fn rejects_traversal() {
        assert!(parse("root:\n  ../evil:\n").is_err());
        assert!(parse("root:\n  a/b:\n").is_err());
    }

    #[test]
    fn rejects_multiple_roots() {
        assert!(parse("a:\nb:\n").is_err());
    }

    #[test]
    fn rejects_non_string_content() {
        assert!(parse("root:\n  n: 5\n").is_err());
    }

    #[test]
    fn render_limits_depth() {
        let t = parse("root:\n  a:\n    b:\n").unwrap();
        assert!(t.render("x", Some(1)).contains('…'));
        assert!(!t.render("x", None).contains('…'));
    }
}
