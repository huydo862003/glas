//! GitHub provider: reads tokens from the gh CLI config, creates repos via API

use std::collections::HashMap;
use std::fs;

use serde::Deserialize;

use super::{home_dir, GitProvider};

pub struct GitHub {
  pub host: String,
}

impl GitProvider for GitHub {
  fn read_cli_token(&self) -> Option<String> {
    let path = home_dir()?.join(".config/gh/hosts.yml");
    let content = fs::read_to_string(path).ok()?;
    let hosts: HashMap<String, GhHost> = serde_yaml::from_str(&content).ok()?;
    hosts.get(&self.host)?.oauth_token.clone()
  }

  fn ensure_repo_exists(
    &self,
    token: &str,
    user: &str,
    repo: &str,
    private: bool,
  ) -> anyhow::Result<()> {
    let api = format!("https://{}/api/v3", self.host);
    let client = reqwest::blocking::Client::new();

    // Try user repo first; if 404, try org
    let check = client
      .get(format!("{api}/repos/{user}/{repo}"))
      .bearer_auth(token)
      .header("User-Agent", "glas")
      .send()?;

    if check.status().is_success() {
      return Ok(());
    }

    // Determine whether `user` is the authenticated user or an org
    let me: serde_json::Value = client
      .get(format!("{api}/user"))
      .bearer_auth(token)
      .header("User-Agent", "glas")
      .send()?
      .json()?;

    let login = me["login"].as_str().unwrap_or("");
    let create_url = if login.eq_ignore_ascii_case(user) {
      format!("{api}/user/repos")
    } else {
      format!("{api}/orgs/{user}/repos")
    };

    let body = serde_json::json!({ "name": repo, "private": private });
    let resp = client
      .post(&create_url)
      .bearer_auth(token)
      .header("User-Agent", "glas")
      .json(&body)
      .send()?;

    let status = resp.status();
    if status.is_success() || status.as_u16() == 422 {
      // 422 Unprocessable Entity = already exists
      return Ok(());
    }

    let text = resp.text().unwrap_or_default();
    anyhow::bail!("GitHub create repo failed ({status}): {text}");
  }
}

// ~/.config/gh/hosts.yml: { "github.com": { oauth_token: "gho_..." } }
#[derive(Deserialize)]
struct GhHost {
  oauth_token: Option<String>,
}
