//! Gitea/Forgejo provider: reads tokens from the tea CLI config, creates repos via API

use std::fs;

use serde::Deserialize;

use super::{GitProvider, home_dir};

pub struct Gitea {
  pub base_url: String,
}

impl GitProvider for Gitea {
  fn read_cli_token(&self) -> Option<String> {
    let path = home_dir()?.join(".tea/config.yml");
    let content = fs::read_to_string(path).ok()?;
    let config: TeaConfig = serde_yaml::from_str(&content).ok()?;
    let target = self.base_url.trim_end_matches('/');
    config
      .logins?
      .into_iter()
      .find(|login| login.url.trim_end_matches('/') == target)?
      .token
  }

  fn ensure_repo_exists(
    &self,
    token: &str,
    user: &str,
    repo: &str,
    private: bool,
  ) -> anyhow::Result<()> {
    let api = format!("{}/api/v1", self.base_url.trim_end_matches('/'));
    let client = reqwest::blocking::Client::new();

    // Check existence
    let check = client
      .get(format!("{api}/repos/{user}/{repo}"))
      .bearer_auth(token)
      .header("User-Agent", "glas")
      .send()?;

    if check.status().is_success() {
      return Ok(());
    }

    // Determine whether user is the authenticated account or an org
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
    if status.is_success() || status.as_u16() == 409 {
      // 409 Conflict = already exists
      return Ok(());
    }

    let text = resp.text().unwrap_or_default();
    anyhow::bail!("Gitea create repo failed ({status}): {text}");
  }
}

// ~/.tea/config.yml: { logins: [{ url: "https://...", token: "..." }] }
#[derive(Deserialize)]
struct TeaConfig {
  logins: Option<Vec<TeaLogin>>,
}

#[derive(Deserialize)]
struct TeaLogin {
  url: String,
  token: Option<String>,
}
