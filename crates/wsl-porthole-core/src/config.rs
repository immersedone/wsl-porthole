//! JSON configuration persistence for rules.

use crate::rules::Rule;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

const CONFIG_VERSION: u32 = 1;

#[derive(Debug, Serialize, Deserialize)]
pub struct RuleConfig {
    pub version: u32,
    #[serde(default)]
    pub distro: String,
    pub rules: Vec<Rule>,
}

impl Default for RuleConfig {
    fn default() -> Self {
        Self {
            version: CONFIG_VERSION,
            distro: "auto".into(),
            rules: Vec::new(),
        }
    }
}

/// Load rules from a JSON config file. Returns an empty config if the file doesn't exist.
pub fn load_rules(path: &Path) -> Result<RuleConfig> {
    if !path.exists() {
        return Ok(RuleConfig::default());
    }
    let contents = std::fs::read_to_string(path)?;
    let config: RuleConfig = serde_json::from_str(&contents)?;
    Ok(config)
}

/// Save rules to a JSON config file (pretty-printed).
pub fn save_rules(path: &Path, config: &RuleConfig) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(config)?;
    std::fs::write(path, json)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::{Direction, Rule};

    #[test]
    fn test_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test-rules.json");

        let mut config = RuleConfig::default();
        config.rules.push(Rule::new("HTTP", Direction::WinToWsl, 80, 80));
        config.rules.push(Rule::new("SSH", Direction::WinToWsl, 22, 22));

        save_rules(&path, &config).unwrap();
        let loaded = load_rules(&path).unwrap();

        assert_eq!(loaded.version, CONFIG_VERSION);
        assert_eq!(loaded.rules.len(), 2);
        assert_eq!(loaded.rules[0].name, "HTTP");
        assert_eq!(loaded.rules[1].name, "SSH");
    }

    #[test]
    fn test_load_missing_file() {
        let path = Path::new("/tmp/does-not-exist-wsl-porthole.json");
        let config = load_rules(path).unwrap();
        assert!(config.rules.is_empty());
    }
}
