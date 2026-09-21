pub mod bitbucket;
pub mod gitea;
pub mod github;
pub mod gitlab;

pub use bitbucket::Bitbucket;
pub use gitea::Gitea;
pub use github::GitHub;
pub use gitlab::GitLab;

use std::path::PathBuf;

pub trait GitProvider: Send + Sync {
  /// Read a token from the provider's installed CLI tool config
  fn read_cli_token(&self) -> Option<String>;

  /// Ensure a repo exists on the provider, creating it if absent
  /// `user` may be a personal account or an org/workspace name
  /// Returns Ok if the repo was created or already existed
  fn ensure_repo_exists(
    &self,
    token: &str,
    user: &str,
    repo: &str,
    private: bool,
  ) -> anyhow::Result<()>;
}

pub(super) fn home_dir() -> Option<PathBuf> {
  std::env::var("HOME").ok().map(Into::into)
}
