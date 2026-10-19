use std::fs;
use std::io;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemePreference {
    Auto,
    Dark,
    Light,
}

impl Default for ThemePreference {
    fn default() -> Self {
        Self::Auto
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub ascii_mode_default: bool,
    pub pinyin_fallback: bool,
    pub candidate_count: usize,
    pub theme: ThemePreference,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ascii_mode_default: false,
            pinyin_fallback: true,
            candidate_count: 6,
            theme: ThemePreference::Auto,
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let Some(path) = config_path() else {
            return Self::default();
        };
        let Ok(text) = fs::read_to_string(&path) else {
            return Self::default();
        };
        let mut config: Self = match toml::from_str(&text) {
            Ok(config) => config,
            Err(err) => {
                eprintln!(
                    "invalid config at {}: {err}; using defaults",
                    path.display()
                );
                Self::default()
            }
        };
        config.normalize();
        config
    }

    pub fn save(&self) -> io::Result<PathBuf> {
        let path = config_path()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no XDG_CONFIG_HOME or HOME"))?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let text = toml::to_string_pretty(self)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
        let tmp = path.with_extension("toml.tmp");
        fs::write(&tmp, text)?;
        fs::rename(&tmp, &path)?;
        Ok(path)
    }

    pub fn normalize(&mut self) {
        self.candidate_count = self.candidate_count.clamp(1, 9);
    }

    pub fn get(&self, key: &str) -> Option<String> {
        match key {
            "ascii_mode_default" | "ascii" => Some(self.ascii_mode_default.to_string()),
            "pinyin_fallback" | "pinyin" => Some(self.pinyin_fallback.to_string()),
            "candidate_count" | "candidates" => Some(self.candidate_count.to_string()),
            "theme" => Some(
                match self.theme {
                    ThemePreference::Auto => "auto",
                    ThemePreference::Dark => "dark",
                    ThemePreference::Light => "light",
                }
                .to_string(),
            ),
            _ => None,
        }
    }

    pub fn apply(&mut self, key: &str, value: &str) -> Result<(), String> {
        match key {
            "ascii_mode_default" | "ascii" => {
                self.ascii_mode_default = parse_bool(value)?;
            }
            "pinyin_fallback" | "pinyin" => {
                self.pinyin_fallback = parse_bool(value)?;
            }
            "candidate_count" | "candidates" => {
                let count: usize = value
                    .parse()
                    .map_err(|_| format!("invalid candidate_count: {value}"))?;
                if !(1..=9).contains(&count) {
                    return Err("candidate_count must be 1..=9".to_string());
                }
                self.candidate_count = count;
            }
            "theme" => {
                self.theme = match value {
                    "auto" => ThemePreference::Auto,
                    "dark" => ThemePreference::Dark,
                    "light" => ThemePreference::Light,
                    _ => return Err(format!("invalid theme: {value}")),
                };
            }
            _ => return Err(format!("unknown key: {key}")),
        }
        Ok(())
    }
}

fn parse_bool(value: &str) -> Result<bool, String> {
    match value {
        "1" | "true" | "yes" | "on" => Ok(true),
        "0" | "false" | "no" | "off" => Ok(false),
        _ => Err(format!("invalid boolean: {value}")),
    }
}

pub fn config_path() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
    Some(base.join("rpci/config.toml"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_stable() {
        let config = Config::default();
        assert!(!config.ascii_mode_default);
        assert!(config.pinyin_fallback);
        assert_eq!(config.candidate_count, 6);
        assert_eq!(config.theme, ThemePreference::Auto);
    }

    #[test]
    fn apply_valid_values() {
        let mut config = Config::default();
        config.apply("ascii", "on").unwrap();
        config.apply("pinyin", "off").unwrap();
        config.apply("candidates", "3").unwrap();
        config.apply("theme", "light").unwrap();
        assert!(config.ascii_mode_default);
        assert!(!config.pinyin_fallback);
        assert_eq!(config.candidate_count, 3);
        assert_eq!(config.theme, ThemePreference::Light);
    }

    #[test]
    fn reject_invalid_values() {
        let mut config = Config::default();
        assert!(config.apply("theme", "blue").is_err());
        assert!(config.apply("candidate_count", "0").is_err());
        assert!(config.apply("candidate_count", "10").is_err());
        assert!(config.apply("unknown", "1").is_err());
    }

    #[test]
    fn normalize_clamps_candidates() {
        let mut config = Config::default();
        config.candidate_count = 99;
        config.normalize();
        assert_eq!(config.candidate_count, 9);
        config.candidate_count = 0;
        config.normalize();
        assert_eq!(config.candidate_count, 1);
    }

    #[test]
    fn toml_roundtrip() {
        let mut config = Config::default();
        config.apply("theme", "dark").unwrap();
        config.apply("candidates", "4").unwrap();
        let text = toml::to_string_pretty(&config).unwrap();
        let parsed: Config = toml::from_str(&text).unwrap();
        assert_eq!(parsed, config);
    }

    #[test]
    fn missing_fields_use_defaults() {
        let parsed: Config = toml::from_str("theme = \"light\"\n").unwrap();
        assert_eq!(parsed.theme, ThemePreference::Light);
        assert_eq!(parsed.candidate_count, 6);
        assert!(parsed.pinyin_fallback);
    }
}
