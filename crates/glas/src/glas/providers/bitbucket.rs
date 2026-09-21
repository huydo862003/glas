//! Bitbucket provider: reads tokens from the bb CLI config, creates repos via API

use std::fs;

use serde::Deserialize;

use super::{home_dir, GitProvider};

pub struct Bitbucket;

impl GitProvider for Bitbucket {
  fn read_cli_token(&self) -> Option<String> {
    let path = home_dir()?.join(".config/bb/config.toml");
    let content = fs::read_to_string(path).ok()?;
    let config: BbConfig = toml::from_str(&content).ok()?;
    config.auth?.token
  }

  fn ensure_repo_exists(
    &self,
    token: &str,
    user: &str,
    repo: &str,
    private: bool,
  ) -> anyhow::Result<()> {
    let api = "https://api.bitbucket.org/2.0";
    let client = reqwest::blocking::Client::new();

    // Check existence
    let check = client
      .get(format!("{api}/repositories/{user}/{repo}"))
      .bearer_auth(token)
      .header("User-Agent", "glas")
      .send()?;

    if check.status().is_success() {
      return Ok(());
    }

    let body = serde_json::json!({
      "scm": "git",
      "is_private": private,
    });

    let resp = client
      .post(format!("{api}/repositories/{user}/{repo}"))
      .bearer_auth(token)
      .header("User-Agent", "glas")
      .json(&body)
      .send()?;

    let status = resp.status();
    if status.is_success() {
      return Ok(());
    }

    let text = resp.text().unwrap_or_default();
    anyhow::bail!("Bitbucket create repo failed ({status}): {text}");
  }
}

// ~/.config/bb/config.toml: [auth] token = "..."
#[derive(Deserialize)]
struct BbConfig {
  auth: Option<BbAuth>,
}

#[derive(Deserialize)]
struct BbAuth {
  token: Option<String>,
}
