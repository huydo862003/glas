//! Raw config types for parsing/saving TOML

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawWorkspaceConfig {
  #[serde(default)]
  pub meta: RawWorkspaceMeta,
  #[serde(default)]
  pub remotes: HashMap<String, RawRemoteConfig>,
  #[serde(default)]
  pub repo: HashMap<String, RawGitRepository>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawWorkspaceMeta {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub repo: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub primary: Option<RawGitRemote>,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub remotes: Vec<RawGitRemote>,
}

/// A global remote definition with URL and default user
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawRemoteConfig {
  pub url: String,
  pub user: String,
}

/// A per-repo or per-meta remote reference (points to a global remote by name)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawGitRemote {
  pub name: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub repo: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub user: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawGitRepository {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub primary: Option<RawGitRemote>,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub remotes: Vec<RawGitRemote>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawSecretsFile {
  #[serde(default)]
  pub remotes: HashMap<String, RawRemoteSecret>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawRemoteSecret {
  pub token: Option<String>,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse_minimal_config() {
    let toml = "";
    let config: RawWorkspaceConfig = toml::from_str(toml).unwrap();
    assert!(config.remotes.is_empty());
    assert!(config.repo.is_empty());
  }

  #[test]
  fn parse_config_with_remote() {
    let toml = r#"
[remotes.github]
url = "https://github.com"
user = "myorg"
"#;
    let config: RawWorkspaceConfig = toml::from_str(toml).unwrap();
    assert_eq!(config.remotes["github"].url, "https://github.com");
    assert_eq!(config.remotes["github"].user, "myorg");
  }

  #[test]
  fn parse_config_with_repo_and_primary() {
    let toml = r#"
[remotes.github]
url = "https://github.com"
user = "myorg"

[repo.myapp]
primary = { name = "github" }
"#;
    let config: RawWorkspaceConfig = toml::from_str(toml).unwrap();
    let repo = &config.repo["myapp"];
    assert_eq!(repo.primary.as_ref().unwrap().name, "github");
  }

  #[test]
  fn reject_unknown_fields_in_remote() {
    let toml = r#"
[remotes.github]
url = "https://github.com"
user = "myorg"
extra = "bad"
"#;
    assert!(toml::from_str::<RawWorkspaceConfig>(toml).is_err());
  }

  #[test]
  fn reject_unknown_fields_at_root() {
    let toml = r#"
unknown_key = "bad"
"#;
    assert!(toml::from_str::<RawWorkspaceConfig>(toml).is_err());
  }

  #[test]
  fn parse_secrets_file() {
    let toml = r#"
[remotes.github]
token = "ghp_xxx"
"#;
    let secrets: RawSecretsFile = toml::from_str(toml).unwrap();
    assert_eq!(secrets.remotes["github"].token.as_deref(), Some("ghp_xxx"));
  }
}
