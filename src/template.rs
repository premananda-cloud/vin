//! Template discovery: built-in templates (embedded in the binary) plus user templates
//! from `<config dir>/templates/*.yaml`. A user template with the same name as a
//! built-in one overrides it.

use crate::config;
use crate::tree::{self, Tree};
use anyhow::{bail, Context, Result};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Origin {
    Builtin,
    User(PathBuf),
}

impl Origin {
    pub fn label(&self) -> &'static str {
        match self {
            Origin::Builtin => "built-in",
            Origin::User(_) => "custom",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Template {
    pub name: String,
    pub description: String,
    pub origin: Origin,
    pub yaml: String,
}

impl Template {
    pub fn tree(&self) -> Result<Tree> {
        tree::parse(&self.yaml).with_context(|| format!("template `{}` is invalid", self.name))
    }
}

/// (name, description, yaml)
const BUILTIN: &[(&str, &str, &str)] = &[
    (
        "ai_ml",
        "AI / ML research: data, configs, experiments, results, reproduction",
        include_str!("directories_structure/ai_ml.yaml"),
    ),
    (
        "econometrics",
        "Empirical economics: chronology, estimation, robustness, replication",
        include_str!("directories_structure/econometrics.yaml"),
    ),
    (
        "physics",
        "Physics: model, derivations, simulation, analysis",
        include_str!("directories_structure/physics.yaml"),
    ),
];

pub fn builtin() -> Vec<Template> {
    BUILTIN
        .iter()
        .map(|(name, desc, yaml)| Template {
            name: (*name).to_string(),
            description: (*desc).to_string(),
            origin: Origin::Builtin,
            yaml: (*yaml).to_string(),
        })
        .collect()
}

/// All templates: built-ins first, then custom ones (custom overrides built-in by name).
pub fn all() -> Result<Vec<Template>> {
    let mut templates = builtin();
    let dir = config::templates_dir()?;
    if dir.is_dir() {
        let mut files: Vec<PathBuf> = fs::read_dir(&dir)?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| {
                matches!(
                    p.extension().and_then(|e| e.to_str()),
                    Some("yaml") | Some("yml")
                )
            })
            .collect();
        files.sort();
        for path in files {
            let Some(name) = path.file_stem().and_then(|s| s.to_str()).map(String::from) else {
                continue;
            };
            let yaml = fs::read_to_string(&path)
                .with_context(|| format!("reading {}", path.display()))?;
            let description = describe_custom(&yaml);
            let t = Template {
                name: name.clone(),
                description,
                origin: Origin::User(path),
                yaml,
            };
            match templates.iter_mut().find(|x| x.name == name) {
                Some(slot) => *slot = t,
                None => templates.push(t),
            }
        }
    }
    Ok(templates)
}

/// Use the first `# description:` comment line if present, else a generic label.
fn describe_custom(yaml: &str) -> String {
    yaml.lines()
        .find_map(|l| l.trim().strip_prefix("# description:"))
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "Custom template".to_string())
}

/// Find a template by exact name, or by unique prefix (`phys` -> `physics`).
pub fn resolve<'a>(query: &str, templates: &'a [Template]) -> Result<&'a Template> {
    let q = query.trim().to_lowercase();
    if let Some(t) = templates.iter().find(|t| t.name.to_lowercase() == q) {
        return Ok(t);
    }
    let matches: Vec<&Template> = templates
        .iter()
        .filter(|t| t.name.to_lowercase().starts_with(&q))
        .collect();
    match matches.as_slice() {
        [one] => Ok(one),
        [] => bail!(
            "unknown template `{query}`. Available: {}",
            names(templates)
        ),
        many => bail!(
            "template `{query}` is ambiguous: matches {}",
            many.iter()
                .map(|t| t.name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn names(templates: &[Template]) -> String {
    templates
        .iter()
        .map(|t| t.name.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtins_all_parse() {
        for t in builtin() {
            let tree = t.tree().unwrap_or_else(|e| panic!("{}: {e:#}", t.name));
            assert!(tree.stats().dirs > 5, "{} looks empty", t.name);
        }
    }

    #[test]
    fn resolves_by_prefix() {
        let all = builtin();
        assert_eq!(resolve("ai", &all).unwrap().name, "ai_ml");
        assert_eq!(resolve("phys", &all).unwrap().name, "physics");
        assert_eq!(resolve("econ", &all).unwrap().name, "econometrics");
        assert!(resolve("nope", &all).is_err());
    }
}
