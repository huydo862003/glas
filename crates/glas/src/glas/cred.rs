//! On-demand credential loading
//!
//! Tokens are never stored in memory at rest
//! `Credential::read` fetches fresh on each call and returns a `SecretString` that zeroes on drop
//!
//! Fallback order: env var, secrets file, git config, provider CLI

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use secrecy::{ExposeSecret, SecretString};

use crate::glas::providers::GitProvider;
use crate::glas::types::RawSecretsFile;

pub struct Credential {
  provider: Option<Box<dyn GitProvider>>,
  secrets_path: Option<PathBuf>,
  global_secrets_path: Option<PathBuf>,
  key: String,
  env_var: Option<String>,
}

impl Credential {
  pub fn new(
    provider: Option<Box<dyn GitProvider>>,
    secrets_path: Option<PathBuf>,
    global_secrets_path: Option<PathBuf>,
    key: String,
    env_var: Option<String>,
  ) -> Self {
    Credential { provider, secrets_path, global_secrets_path, key, env_var }
  }

  /// Ensure the repo exists on the provider, creating it if absent
  /// No-op when no provider is configured for this remote
  pub fn ensure_repo_exists(&self, user: &str, repo: &str, private: bool) -> anyhow::Result<()> {
    let Some(provider) = &self.provider else {
      return Ok(());
    };
    let token = self.read()?;
    provider.ensure_repo_exists(token.expose_secret(), user, repo, private)
  }

  pub fn read(&self) -> anyhow::Result<SecretString> {
    // 1. Env var
    if let Some(env_var) = &self.env_var
      && let Ok(val) = env::var(env_var)
    {
      return Ok(SecretString::from(val));
    }

    // 2. Workspace secrets file
    if let Some(path) = &self.secrets_path
      && path.exists()
      && let Ok(token) = read_token_from_file(path, &self.key)
    {
      return Ok(SecretString::from(token));
    }

    // 3. Global secrets file
    if let Some(path) = &self.global_secrets_path
      && path.exists()
      && let Ok(token) = read_token_from_file(path, &self.key)
    {
      return Ok(SecretString::from(token));
    }

    // 4. Git config: glas.remote.<name>.token
    let git_config_key = format!("glas.remote.{}.token", self.key);
    if let Some(val) = read_git_config(&git_config_key) {
      return Ok(SecretString::from(val));
    }

    // 5. Provider CLI tool config (gh, glab, tea, bb)
    if let Some(provider) = &self.provider
      && let Some(val) = provider.read_cli_token()
    {
      return Ok(SecretString::from(val));
    }

    anyhow::bail!(
      "no credential found for '{}' (env var, secrets file, git config, or CLI config)",
      self.key
    )
  }
}

fn read_git_config(key: &str) -> Option<String> {
  let config = git2::Config::open_default().ok()?;
  config.get_string(key).ok()
}

fn read_token_from_file(path: &Path, key: &str) -> anyhow::Result<String> {
  let secrets: RawSecretsFile = toml::from_str(&fs::read_to_string(path)?)?;
  secrets
    .remotes
    .get(key)
    .and_then(|entry| entry.token.clone())
    .ok_or_else(|| anyhow::anyhow!("no token found for '{key}' in secrets file"))
}
