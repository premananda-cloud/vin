//! User configuration: where vin stores its settings and custom templates.
//!
//! Layout (Linux): `~/.config/vin/config.toml` and `~/.config/vin/templates/*.yaml`.
//! Set `VIN_CONFIG_DIR` to use a different directory.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Template used when the user has not chosen a default.
pub const FALLBACK_DEFAULT: &str = "ai_ml";

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_template: Option<String>,
}

pub fn config_dir() -> Result<PathBuf> {
    if let Some(dir) = std::env::var_os("VIN_CONFIG_DIR") {
        return Ok(PathBuf::from(dir));
    }
    dirs::config_dir()
        .map(|d| d.join("vin"))
        .context("could not determine the config directory (set VIN_CONFIG_DIR)")
}

pub fn config_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("config.toml"))
}

pub fn templates_dir() -> Result<PathBuf> {
    Ok(config_dir()?.join("templates"))
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = config_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let text =
            fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
        toml::from_str(&text).with_context(|| format!("parsing {}", path.display()))
    }

    pub fn save(&self) -> Result<()> {
        let path = config_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let text = toml::to_string_pretty(self)?;
        fs::write(&path, text).with_context(|| format!("writing {}", path.display()))
    }

    /// The effective default template name (configured, or the built-in fallback).
    pub fn default_name(&self) -> &str {
        self.default_template.as_deref().unwrap_or(FALLBACK_DEFAULT)
    }
}
