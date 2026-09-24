use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ConfigFile {
    pub api_user: Option<String>,
    pub api_key: Option<String>,
    pub client_ip: Option<String>,
    pub sandbox: Option<bool>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamecheapConfig {
    pub api_user: String,
    pub api_key: String,
    pub client_ip: String,
    pub sandbox: bool,
}

impl NamecheapConfig {
    pub fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .or_else(dirs::home_dir)
            .context("Could not determine user configuration directory")?
            .join("namecheap");
        Ok(config_dir.join("config.json"))
    }

    pub fn load_file() -> ConfigFile {
        let path = match Self::config_path() {
            Ok(p) => p,
            Err(_) => return ConfigFile::default(),
        };

        if !path.exists() {
            return ConfigFile::default();
        }

        match fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => ConfigFile::default(),
        }
    }

    pub fn save_file(config: &ConfigFile) -> Result<PathBuf> {
        let path = Self::config_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create config directory: {}", parent.display())
            })?;
        }
        let content = serde_json::to_string_pretty(config)?;
        fs::write(&path, content)
            .with_context(|| format!("Failed to write configuration to {}", path.display()))?;
        Ok(path)
    }

    pub fn resolve(
        flag_user: Option<String>,
        flag_key: Option<String>,
        flag_ip: Option<String>,
        flag_sandbox: bool,
    ) -> Result<Self> {
        let file_config = Self::load_file();

        let api_user = flag_user
            .or_else(|| std::env::var("NAMECHEAP_API_USER").ok())
            .or(file_config.api_user)
            .unwrap_or_default();

        let api_key = flag_key
            .or_else(|| std::env::var("NAMECHEAP_API_KEY").ok())
            .or(file_config.api_key)
            .unwrap_or_default();

        let client_ip = flag_ip
            .or_else(|| std::env::var("NAMECHEAP_CLIENT_IP").ok())
            .or(file_config.client_ip)
            .unwrap_or_default();

        let sandbox = flag_sandbox
            || std::env::var("NAMECHEAP_SANDBOX")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(false)
            || file_config.sandbox.unwrap_or(false);

        if api_user.trim().is_empty() {
            anyhow::bail!(
                "Namecheap API User is required. Specify with --api-user, NAMECHEAP_API_USER env var, or run 'namecheap-cli config set --api-user <user>'"
            );
        }
        if api_key.trim().is_empty() {
            anyhow::bail!(
                "Namecheap API Key is required. Specify with --api-key, NAMECHEAP_API_KEY env var, or run 'namecheap-cli config set --api-key <key>'"
            );
        }
        if client_ip.trim().is_empty() {
            anyhow::bail!(
                "Client IP is required. Specify with --client-ip, NAMECHEAP_CLIENT_IP env var, or run 'namecheap-cli config set --client-ip <ip>' (Hint: run 'namecheap-cli ip' to view your current public IP)"
            );
        }

        Ok(Self {
            api_user: api_user.trim().to_string(),
            api_key: api_key.trim().to_string(),
            client_ip: client_ip.trim().to_string(),
            sandbox,
        })
    }

    pub fn base_url(&self) -> &'static str {
        if self.sandbox {
            "https://api.sandbox.namecheap.com/xml.response"
        } else {
            "https://api.namecheap.com/xml.response"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_url() {
        let prod = NamecheapConfig {
            api_user: "user".into(),
            api_key: "key".into(),
            client_ip: "1.2.3.4".into(),
            sandbox: false,
        };
        assert_eq!(prod.base_url(), "https://api.namecheap.com/xml.response");

        let sandbox = NamecheapConfig {
            api_user: "user".into(),
            api_key: "key".into(),
            client_ip: "1.2.3.4".into(),
            sandbox: true,
        };
        assert_eq!(
            sandbox.base_url(),
            "https://api.sandbox.namecheap.com/xml.response"
        );
    }
}
