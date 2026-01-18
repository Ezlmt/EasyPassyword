use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::core::PasswordConfig;
use crate::error::{EasyPasswordError, Result};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub default: DefaultConfig,
    #[serde(default)]
    pub sites: HashMap<String, SiteConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultConfig {
    pub master_key: Option<String>,
    #[serde(default = "default_length")]
    pub length: usize,
    #[serde(default = "default_true")]
    pub lowercase: bool,
    #[serde(default = "default_true")]
    pub uppercase: bool,
    #[serde(default = "default_true")]
    pub digits: bool,
    #[serde(default = "default_true")]
    pub symbols: bool,
    #[serde(default = "default_trigger_prefix")]
    pub trigger_prefix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SiteConfig {
    pub length: Option<usize>,
    pub lowercase: Option<bool>,
    pub uppercase: Option<bool>,
    pub digits: Option<bool>,
    pub symbols: Option<bool>,
    pub counter: Option<u32>,
}

fn default_length() -> usize {
    16
}
fn default_true() -> bool {
    true
}
fn default_trigger_prefix() -> String {
    ";;".to_string()
}

impl Default for DefaultConfig {
    fn default() -> Self {
        Self {
            master_key: None,
            length: default_length(),
            lowercase: true,
            uppercase: true,
            digits: true,
            symbols: true,
            trigger_prefix: default_trigger_prefix(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            let config: Config =
                toml::from_str(&content).map_err(EasyPasswordError::TomlParse)?;
            Ok(config)
        } else {
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content =
            toml::to_string_pretty(self).map_err(|e| EasyPasswordError::Config(e.to_string()))?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| EasyPasswordError::Config("Cannot find config directory".to_string()))?;
        Ok(config_dir.join("easypassword").join("config.toml"))
    }

    pub fn get_password_config(&self, site: &str) -> PasswordConfig {
        let site_lower = site.to_lowercase();
        let site_config = self.sites.get(&site_lower);

        PasswordConfig {
            length: site_config
                .and_then(|s| s.length)
                .unwrap_or(self.default.length),
            use_lowercase: site_config
                .and_then(|s| s.lowercase)
                .unwrap_or(self.default.lowercase),
            use_uppercase: site_config
                .and_then(|s| s.uppercase)
                .unwrap_or(self.default.uppercase),
            use_digits: site_config
                .and_then(|s| s.digits)
                .unwrap_or(self.default.digits),
            use_symbols: site_config
                .and_then(|s| s.symbols)
                .unwrap_or(self.default.symbols),
        }
    }

    pub fn get_counter(&self, site: &str) -> u32 {
        let site_lower = site.to_lowercase();
        self.sites
            .get(&site_lower)
            .and_then(|s| s.counter)
            .unwrap_or(1)
    }
}
