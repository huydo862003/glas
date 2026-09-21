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
