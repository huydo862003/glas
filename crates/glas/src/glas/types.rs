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

  // Empty string parses to default config with no remotes or repos
  #[test]
  fn empty_toml_parses_to_default_config() {
    let toml = "";
    let config: RawWorkspaceConfig = toml::from_str(toml).unwrap();
    assert!(config.remotes.is_empty());
    assert!(config.repo.is_empty());
  }

  // Remote section with url and user parses correctly
  #[test]
  fn remote_section_parses_url_and_user() {
    let toml = r#"
[remotes.github]
url = "https://github.com"
user = "myorg"
"#;
    let config: RawWorkspaceConfig = toml::from_str(toml).unwrap();
    assert_eq!(config.remotes["github"].url, "https://github.com");
    assert_eq!(config.remotes["github"].user, "myorg");
  }

  // Repo with inline primary table parses the remote name
  #[test]
  fn repo_with_inline_primary_parses_remote_name() {
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

  // Extra fields in remote section are rejected (deny_unknown_fields)
  #[test]
  fn unknown_field_in_remote_is_rejected() {
    let toml = r#"
[remotes.github]
url = "https://github.com"
user = "myorg"
extra = "bad"
"#;
    assert!(toml::from_str::<RawWorkspaceConfig>(toml).is_err());
  }

  // Extra fields at root level are rejected (deny_unknown_fields)
  #[test]
  fn unknown_field_at_root_is_rejected() {
    let toml = r#"
unknown_key = "bad"
"#;
    assert!(toml::from_str::<RawWorkspaceConfig>(toml).is_err());
  }

  // Secrets file with token parses correctly
  #[test]
  fn secrets_file_parses_token() {
    let toml = r#"
[remotes.github]
token = "ghp_xxx"
"#;
    let secrets: RawSecretsFile = toml::from_str(toml).unwrap();
    assert_eq!(secrets.remotes["github"].token.as_deref(), Some("ghp_xxx"));
  }
}
