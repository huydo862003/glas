use std::collections::{HashMap, HashSet};

use crate::glas::types::{RawGitRemote, RawRemoteConfig, RawWorkspaceConfig};
use crate::types::{GitRemoteProvider, GitRepositoryName, GitUserName};

pub fn validate_config(config: &RawWorkspaceConfig) -> anyhow::Result<()> {
  for (name, remote) in &config.remotes {
    name.parse::<GitRemoteProvider>()?;
    remote.url.parse::<crate::types::GitRemoteUrl>()?;
    remote.user.parse::<GitUserName>()?;
  }

  for name in config.repo.keys() {
    name.parse::<GitRepositoryName>()?;
  }

  if let Some(primary) = &config.meta.primary
    && !config.remotes.contains_key(&primary.name)
  {
    anyhow::bail!(
      "meta: primary remote '{}' not found in [remotes]",
      primary.name
    );
  }
  validate_remote_list("meta", &config.meta.remotes, &config.remotes)?;

  for (repo_name, repo_config) in &config.repo {
    if let Some(primary) = &repo_config.primary
      && !config.remotes.contains_key(&primary.name)
    {
      anyhow::bail!(
        "repo '{repo_name}': primary remote '{}' not found in [remotes]",
        primary.name
      );
    }
    validate_remote_list(repo_name, &repo_config.remotes, &config.remotes)?;
  }
  Ok(())
}

fn validate_remote_list(
  context: &str,
  remotes: &[RawGitRemote],
  global_remotes: &HashMap<String, RawRemoteConfig>,
) -> anyhow::Result<()> {
  let mut seen = HashSet::new();
  for remote in remotes {
    if !global_remotes.contains_key(&remote.name) {
      anyhow::bail!("{context}: remote '{}' not found in [remotes]", remote.name);
    }
    if !seen.insert(&remote.name) {
      anyhow::bail!("{context}: duplicate remote '{}'", remote.name);
    }
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::glas::types::RawGitRepository;

  fn make_config(remotes: &[(&str, &str, &str)]) -> RawWorkspaceConfig {
    let mut config = RawWorkspaceConfig::default();
    for (name, url, user) in remotes {
      config.remotes.insert(
        name.to_string(),
        RawRemoteConfig {
          url: url.to_string(),
          user: user.to_string(),
        },
      );
    }
    config
  }

  // Empty config is valid (no remotes, no repos)
  #[test]
  fn empty_config_is_valid() {
    assert!(validate_config(&RawWorkspaceConfig::default()).is_ok());
  }

  // Config with one well-formed remote passes
  #[test]
  fn single_remote_is_valid() {
    let config = make_config(&[("github", "https://github.com", "myorg")]);
    assert!(validate_config(&config).is_ok());
  }

  // Remote name must not be empty
  #[test]
  fn empty_remote_name_is_rejected() {
    let config = make_config(&[("", "https://github.com", "myorg")]);
    assert!(validate_config(&config).is_err());
  }

  // Remote URL must not be empty
  #[test]
  fn empty_url_is_rejected() {
    let config = make_config(&[("github", "", "myorg")]);
    assert!(validate_config(&config).is_err());
  }

  // Remote user must not be empty
  #[test]
  fn empty_user_is_rejected() {
    let config = make_config(&[("github", "https://github.com", "")]);
    assert!(validate_config(&config).is_err());
  }

  // Repo primary must reference an existing remote
  #[test]
  fn repo_primary_referencing_missing_remote_is_rejected() {
    let mut config = make_config(&[("github", "https://github.com", "myorg")]);
    config.repo.insert(
      "myrepo".to_string(),
      RawGitRepository {
        primary: Some(RawGitRemote {
          name: "gitlab".to_string(),
          repo: None,
          user: None,
        }),
        remotes: vec![],
      },
    );
    assert!(validate_config(&config).is_err());
  }

  // Repo primary referencing an existing remote passes
  #[test]
  fn repo_primary_referencing_existing_remote_is_valid() {
    let mut config = make_config(&[("github", "https://github.com", "myorg")]);
    config.repo.insert(
      "myrepo".to_string(),
      RawGitRepository {
        primary: Some(RawGitRemote {
          name: "github".to_string(),
          repo: None,
          user: None,
        }),
        remotes: vec![],
      },
    );
    assert!(validate_config(&config).is_ok());
  }
}
