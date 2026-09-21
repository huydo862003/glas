use std::fmt;
use std::ops::Deref;
use std::path::PathBuf;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::glas::cred::Credential;

pub struct WorkspaceConfig {
  pub meta: WorkspaceMeta,
  pub remotes: Vec<RemoteConfig>, // Resolved global remotes
  pub repos: Vec<GitRepository>,
}

impl WorkspaceConfig {
  pub fn find_repo(&self, name: &str) -> Option<&GitRepository> {
    self.repos.iter().find(|repo| repo.name == name)
  }
}

/// A resolved global remote (hosting service definition)
pub struct RemoteConfig {
  pub name: GitRemoteProvider,
  pub url: GitRemoteUrl,
  pub user: GitUserName,
  pub cred: Credential,
}

pub struct WorkspaceMeta {
  pub name: GitRepositoryName,
  pub path: PathBuf,
  pub primary: Option<GitRemote>,
  pub push_remotes: Vec<GitRemote>,
}

pub struct GitRepository {
  pub name: GitRepositoryName,
  pub path: PathBuf,
  pub primary: Option<GitRemote>,
  pub push_remotes: Vec<GitRemote>,
}

/// A resolved remote for a specific repo
pub struct GitRemote {
  pub name: GitRemoteProvider,
  pub url: GitRemoteUrl,
  pub user: GitUserName,
  pub cred: Credential,
}

/// Known or custom git hosting provider
#[derive(Clone, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub enum GitRemoteProvider {
  GitHub,
  GitLab,
  Gitea,
  Forgejo,
  Bitbucket,
  Other(String),
}

impl GitRemoteProvider {
  pub fn as_str(&self) -> &str {
    match self {
      Self::GitHub => "github",
      Self::GitLab => "gitlab",
      Self::Gitea => "gitea",
      Self::Forgejo => "forgejo",
      Self::Bitbucket => "bitbucket",
      Self::Other(s) => s.as_str(),
    }
  }
}

impl FromStr for GitRemoteProvider {
  type Err = anyhow::Error;
  fn from_str(s: &str) -> anyhow::Result<Self> {
    if s.is_empty() {
      anyhow::bail!("provider name cannot be empty");
    }
    Ok(match s {
      "github" => Self::GitHub,
      "gitlab" => Self::GitLab,
      "gitea" => Self::Gitea,
      "forgejo" => Self::Forgejo,
      "bitbucket" => Self::Bitbucket,
      other => {
        if other.contains('.') || other.contains('/') {
          anyhow::bail!("provider name '{other}' must not contain '.' or '/'");
        }
        Self::Other(other.to_string())
      }
    })
  }
}

impl TryFrom<String> for GitRemoteProvider {
  type Error = anyhow::Error;
  fn try_from(s: String) -> anyhow::Result<Self> {
    s.parse()
  }
}

impl From<GitRemoteProvider> for String {
  fn from(p: GitRemoteProvider) -> String {
    p.as_str().to_string()
  }
}

impl fmt::Display for GitRemoteProvider {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(self.as_str())
  }
}

impl PartialEq<str> for GitRemoteProvider {
  fn eq(&self, other: &str) -> bool {
    self.as_str() == other
  }
}

impl PartialEq<&str> for GitRemoteProvider {
  fn eq(&self, other: &&str) -> bool {
    self.as_str() == *other
  }
}

/// A git remote URL (base or full)
#[derive(Clone, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct GitRemoteUrl(String);

impl FromStr for GitRemoteUrl {
  type Err = anyhow::Error;
  fn from_str(s: &str) -> anyhow::Result<Self> {
    if s.is_empty() {
      anyhow::bail!("URL cannot be empty");
    }
    Ok(GitRemoteUrl(s.to_string()))
  }
}

impl TryFrom<String> for GitRemoteUrl {
  type Error = anyhow::Error;
  fn try_from(s: String) -> anyhow::Result<Self> {
    s.parse()
  }
}

impl From<GitRemoteUrl> for String {
  fn from(u: GitRemoteUrl) -> String {
    u.0
  }
}

impl Deref for GitRemoteUrl {
  type Target = str;
  fn deref(&self) -> &str {
    &self.0
  }
}

impl fmt::Display for GitRemoteUrl {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.0.fmt(f)
  }
}

/// A git forge username
#[derive(Clone, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct GitUserName(String);

impl FromStr for GitUserName {
  type Err = anyhow::Error;
  fn from_str(s: &str) -> anyhow::Result<Self> {
    if s.is_empty() {
      anyhow::bail!("username cannot be empty");
    }
    if s.contains('/') {
      anyhow::bail!("username '{s}' must not contain '/'");
    }
    Ok(GitUserName(s.to_string()))
  }
}

impl TryFrom<String> for GitUserName {
  type Error = anyhow::Error;
  fn try_from(s: String) -> anyhow::Result<Self> {
    s.parse()
  }
}

impl From<GitUserName> for String {
  fn from(u: GitUserName) -> String {
    u.0
  }
}

impl Deref for GitUserName {
  type Target = str;
  fn deref(&self) -> &str {
    &self.0
  }
}

impl fmt::Display for GitUserName {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.0.fmt(f)
  }
}

/// A repository name (directory name within the workspace)
#[derive(Clone, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct GitRepositoryName(String);

impl GitRepositoryName {
  pub fn as_str(&self) -> &str {
    &self.0
  }
}

impl FromStr for GitRepositoryName {
  type Err = anyhow::Error;
  fn from_str(s: &str) -> anyhow::Result<Self> {
    if s.is_empty() {
      anyhow::bail!("repository name cannot be empty");
    }
    if s.contains('/') {
      anyhow::bail!("repository name '{s}' must not contain '/'");
    }
    Ok(GitRepositoryName(s.to_string()))
  }
}

impl TryFrom<String> for GitRepositoryName {
  type Error = anyhow::Error;
  fn try_from(s: String) -> anyhow::Result<Self> {
    s.parse()
  }
}

impl From<GitRepositoryName> for String {
  fn from(r: GitRepositoryName) -> String {
    r.0
  }
}

impl Deref for GitRepositoryName {
  type Target = str;
  fn deref(&self) -> &str {
    &self.0
  }
}

impl fmt::Display for GitRepositoryName {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.0.fmt(f)
  }
}

impl PartialEq<str> for GitRepositoryName {
  fn eq(&self, other: &str) -> bool {
    self.0 == other
  }
}

impl PartialEq<&str> for GitRepositoryName {
  fn eq(&self, other: &&str) -> bool {
    self.0 == *other
  }
}
