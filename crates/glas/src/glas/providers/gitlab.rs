//! GitLab provider: reads tokens from the glab CLI config, creates repos via API

use std::collections::HashMap;
use std::fs;

use serde::Deserialize;

use super::{home_dir, GitProvider};

pub struct GitLab {
  pub host: String,
}

impl GitProvider for GitLab {
  fn read_cli_token(&self) -> Option<String> {
    let path = home_dir()?.join(".config/glab-cli/config.yml");
    let content = fs::read_to_string(path).ok()?;
    let config: GlabConfig = serde_yaml::from_str(&content).ok()?;
    config.hosts?.get(&self.host)?.token.clone()
  }

  fn ensure_repo_exists(
    &self,
    token: &str,
    user: &str,
    repo: &str,
    private: bool,
  ) -> anyhow::Result<()> {
    let api = format!("https://{}/api/v4", self.host);
    let client = reqwest::blocking::Client::new();

    // Check if project exists: GET /projects/{user}%2F{repo}
    let encoded = format!("{user}%2F{repo}");
    let check = client
      .get(format!("{api}/projects/{encoded}"))
      .header("PRIVATE-TOKEN", token)
      .header("User-Agent", "glas")
      .send()?;

    if check.status().is_success() {
      return Ok(());
    }

    let visibility = if private { "private" } else { "public" };
    let body = serde_json::json!({
      "name": repo,
      "path": repo,
      "visibility": visibility,
      "namespace_path": user,
    });

    let resp = client
      .post(format!("{api}/projects"))
      .header("PRIVATE-TOKEN", token)
      .header("User-Agent", "glas")
      .json(&body)
      .send()?;

    let status = resp.status();
    if status.is_success() {
      return Ok(());
    }
    // 400 with "has already been taken" is treated as already existing
    if status.as_u16() == 400 {
      let text = resp.text().unwrap_or_default();
      if text.contains("has already been taken") {
        return Ok(());
      }
      anyhow::bail!("GitLab create repo failed ({status}): {text}");
    }

    let text = resp.text().unwrap_or_default();
    anyhow::bail!("GitLab create repo failed ({status}): {text}");
  }
}

// ~/.config/glab-cli/config.yml: { hosts: { "gitlab.com": { token: "glpat-..." } } }
#[derive(Deserialize)]
struct GlabConfig {
  hosts: Option<HashMap<String, GlabHost>>,
}

#[derive(Deserialize)]
struct GlabHost {
  token: Option<String>,
}
