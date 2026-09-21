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

  fn make_config(remotes: &[(&str, &str, &str)]) -> RawWorkspaceConfig {
    let mut config = RawWorkspaceConfig::default();
    for (name, url, user) in remotes {
      config.remotes.insert(
        name.to_string(),
        RawRemoteConfig { url: url.to_string(), user: user.to_string() },
      );
    }
    config
  }

  #[test]
  fn valid_empty_config() {
    assert!(validate_config(&RawWorkspaceConfig::default()).is_ok());
  }

  #[test]
  fn valid_config_with_remote() {
    let config = make_config(&[("github", "https://github.com", "myorg")]);
    assert!(validate_config(&config).is_ok());
  }

  #[test]
  fn reject_empty_remote_name() {
    let config = make_config(&[("", "https://github.com", "myorg")]);
    assert!(validate_config(&config).is_err());
  }

  #[test]
  fn reject_empty_url() {
    let config = make_config(&[("github", "", "myorg")]);
    assert!(validate_config(&config).is_err());
  }

  #[test]
  fn reject_empty_user() {
    let config = make_config(&[("github", "https://github.com", "")]);
    assert!(validate_config(&config).is_err());
  }

  #[test]
  fn reject_repo_with_missing_primary_remote() {
    let mut config = make_config(&[("github", "https://github.com", "myorg")]);
    config.repo.insert("myrepo".to_string(), crate::glas::types::RawGitRepository {
      primary: Some(RawGitRemote { name: "gitlab".to_string(), repo: None, user: None }),
      remotes: vec![],
    });
    assert!(validate_config(&config).is_err());
  }

  #[test]
  fn accept_repo_with_valid_primary() {
    let mut config = make_config(&[("github", "https://github.com", "myorg")]);
    config.repo.insert("myrepo".to_string(), crate::glas::types::RawGitRepository {
      primary: Some(RawGitRemote { name: "github".to_string(), repo: None, user: None }),
      remotes: vec![],
    });
    assert!(validate_config(&config).is_ok());
  }
}
