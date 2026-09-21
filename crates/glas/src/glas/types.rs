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
