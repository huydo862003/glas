//! The `.glas/` workspace

pub mod cred;
pub mod providers;
mod constants;
mod resolve;
mod types;
mod validate;

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::git;
use crate::glas::constants::{
  CONFIG_FILE, CONFIG_HEADER, GLAS_DIR, GLAS_LOCAL_PATH, GLOBAL_CONFIG_SUBDIR,
  GLOBAL_SECRETS_FILE, LOCAL_GITIGNORE, SECRETS_HEADER,
};
use crate::glas::resolve::resolve_config;
use crate::glas::types::{
  RawGitRemote, RawRemoteConfig, RawSecretsFile, RawWorkspaceConfig,
  RawWorkspaceMeta,
};
use crate::glas::validate::validate_config;
use crate::types::WorkspaceConfig;

pub struct Workspace {
  root: PathBuf,
  config: RawWorkspaceConfig,
  global_secrets_path: Option<PathBuf>,
}

impl Workspace {
  /// Find the workspace root, load and validate its config
  pub fn load() -> anyhow::Result<Workspace> {
    // Walk up from cwd to find the nearest `.glas/` directory
    let root = if let Ok(dir) = std::env::var("GLAS_DIR") {
      // Env var overrides for testing
      PathBuf::from(dir)
    } else {
      let mut current = std::env::current_dir()?;
      loop {
        if current.join(GLAS_DIR).is_dir() {
          break current;
        }
        match current.parent() {
          Some(parent) => current = parent.to_path_buf(),
          None => anyhow::bail!(
            r#"not inside a glas workspace (no .glas/ found)
hint: run `glas init` to create one"#
          ),
        }
      }
    };

    let config_path = root.join(GLAS_DIR).join(CONFIG_FILE);
    // Default to empty config if the file does not exist yet
    let mut config: RawWorkspaceConfig = if config_path.exists() {
      toml::from_str(&fs::read_to_string(&config_path)?)?
    } else {
      RawWorkspaceConfig::default()
    };

    // Load global config and merge: workspace remotes override global ones by name
    let (global_remotes, global_secrets_path) = load_global()?;
    for (name, remote) in global_remotes {
      config.remotes.entry(name).or_insert(remote);
    }

    validate_config(&config)?; // Callers can assume the config is valid

    Ok(Workspace { root, config, global_secrets_path })
  }

  /// Resolve raw config into fully-computed paths and URLs
  pub fn config(&self) -> WorkspaceConfig {
    resolve_config(&self.config, &self.root, self.global_secrets_path.as_deref())
  }

  /// Return the workspace root path
  pub fn root(&self) -> &Path {
    &self.root
  }

  /// Infer the current repo name from the working directory name
  pub fn get_current_repo_name(&self) -> anyhow::Result<String> {
    let cwd = std::env::current_dir()?;
    cwd
      .file_name()
      .and_then(|file_name| file_name.to_str())
      .map(|str_name| str_name.to_string())
      .ok_or_else(|| anyhow::anyhow!("could not determine repo name from current directory"))
  }

  /// Register a new global remote
  pub fn add_remote(&mut self, name: String, url: String, user: String, force: bool) -> anyhow::Result<()> {
    if !force && self.config.remotes.contains_key(&name) {
      anyhow::bail!("remote '{name}' already exists");
    }
    self.config.remotes.insert(name, RawRemoteConfig { url, user });
    Ok(())
  }

  /// Remove a global remote, refusing if any repo still references it
  pub fn remove_remote(&mut self, name: &str) -> anyhow::Result<()> {
    // Removal would silently break any repo that references this remote
    let refs: Vec<&str> = self
      .config
      .repo
      .iter()
      .filter(|(_, repo)| {
        repo
          .primary
          .as_ref()
          .is_some_and(|primary| primary.name == name)
          || repo.remotes.iter().any(|remote| remote.name == name)
      })
      .map(|(repo_name, _)| repo_name.as_str())
      .collect();

    if !refs.is_empty() {
      anyhow::bail!(
        "remote '{name}' is still referenced by repos: {}",
        refs.join(", ")
      );
    }

    if self.config.remotes.remove(name).is_none() {
      anyhow::bail!("remote '{name}' not found");
    }
    Ok(())
  }

  /// Track a repo in the workspace
  pub fn add_repo(&mut self, name: String, force: bool) -> anyhow::Result<()> {
    if !force && self.config.repo.contains_key(&name) {
      anyhow::bail!("repo '{name}' is already tracked");
    }
    self.config.repo.insert(name, Default::default());
    Ok(())
  }

  /// Stop tracking a repo
  pub fn remove_repo(&mut self, name: &str) -> anyhow::Result<()> {
    if self.config.repo.remove(name).is_none() {
      anyhow::bail!("repo '{name}' is not tracked");
    }
    Ok(())
  }

  /// Add a push remote on a repo
  pub fn add_repo_remote(
    &mut self,
    repo_name: &str,
    remote_name: String,
    repo_override: Option<String>,
    user_override: Option<String>,
    force: bool,
  ) -> anyhow::Result<()> {
    if !self.config.remotes.contains_key(&remote_name) {
      anyhow::bail!("remote '{remote_name}' not found - add it with `glas remote add`");
    }

    let repo = self
      .config
      .repo
      .get_mut(repo_name)
      .ok_or_else(|| anyhow::anyhow!("repo '{repo_name}' is not tracked"))?;

    if !force && repo.remotes.iter().any(|existing| existing.name == remote_name) {
      anyhow::bail!("remote '{remote_name}' is already set on repo '{repo_name}'");
    }
    repo.remotes.retain(|existing| existing.name != remote_name);
    repo.remotes.push(RawGitRemote {
      name: remote_name,
      repo: repo_override,
      user: user_override,
    });
    Ok(())
  }

  /// Remove a push remote from a repo
  pub fn remove_repo_remote(&mut self, repo_name: &str, remote_name: &str) -> anyhow::Result<()> {
    let repo = self
      .config
      .repo
      .get_mut(repo_name)
      .ok_or_else(|| anyhow::anyhow!("repo '{repo_name}' is not tracked"))?;

    let before = repo.remotes.len();
    repo.remotes.retain(|existing| existing.name != remote_name);

    if repo.remotes.len() == before {
      // No change means the remote was never listed
      anyhow::bail!("remote '{remote_name}' not found on repo '{repo_name}'");
    }
    Ok(())
  }

  /// Set which remote is used as the pull source for a repo
  pub fn set_repo_primary(
    &mut self,
    repo_name: &str,
    remote_name: String,
    repo_override: Option<String>,
    user_override: Option<String>,
  ) -> anyhow::Result<()> {
    if !self.config.remotes.contains_key(&remote_name) {
      // Must exist to be resolvable
      anyhow::bail!("remote '{remote_name}' not found - add it with `glas remote add`");
    }

    let repo = self
      .config
      .repo
      .get_mut(repo_name)
      .ok_or_else(|| anyhow::anyhow!("repo '{repo_name}' is not tracked"))?;

    repo.primary = Some(RawGitRemote {
      name: remote_name,
      repo: repo_override,
      user: user_override,
    });
    Ok(())
  }

  /// Write a token for a remote to the global secrets file (shared across workspaces)
  pub fn write_global_token(&self, remote_name: &str, token: String) -> anyhow::Result<()> {
    let Some(home) = std::env::var("HOME").ok().map(PathBuf::from) else {
      anyhow::bail!("HOME is not set");
    };
    let dir = home.join(GLOBAL_CONFIG_SUBDIR);
    let path = dir.join(GLOBAL_SECRETS_FILE);
    self.write_token_to(&path, remote_name, token)
  }

  /// Write a token for a global remote to the workspace-local secrets file
  pub fn write_token(&self, remote_name: &str, token: String) -> anyhow::Result<()> {
    let path = self.root.join(GLAS_LOCAL_PATH).join("secrets.toml");
    self.write_token_to(&path, remote_name, token)
  }

  fn write_token_to(&self, path: &Path, remote_name: &str, token: String) -> anyhow::Result<()> {
    let mut secrets: RawSecretsFile = if path.exists() {
      toml::from_str(&fs::read_to_string(path)?)?
    } else {
      RawSecretsFile::default()
    };

    secrets.remotes.entry(remote_name.to_string()).or_default().token = Some(token);

    if let Some(parent) = path.parent() {
      fs::create_dir_all(parent)?;
    }
    write_secrets_file(path, &format!("{SECRETS_HEADER}{}", toml::to_string_pretty(&secrets)?))?;
    Ok(())
  }

  /// Create the `.glas/` directory structure and initialise it as a git repo
  pub fn init(workspace: &Path, meta_repo: &str) -> anyhow::Result<()> {
    let glas_dir = workspace.join(GLAS_DIR);

    if glas_dir.exists() {
      anyhow::bail!("workspace already initialized at {}", workspace.display());
    }

    let local_dir = workspace.join(GLAS_LOCAL_PATH); // Holds secrets, gitignored

    fs::create_dir_all(&local_dir)?;

    let gitignore = local_dir.join(".gitignore"); // Excludes everything except itself
    if !gitignore.exists() {
      fs::write(&gitignore, LOCAL_GITIGNORE)?;
    }

    let config_path = glas_dir.join(CONFIG_FILE);
    let config = RawWorkspaceConfig {
      meta: RawWorkspaceMeta {
        repo: Some(meta_repo.to_string()),
        ..Default::default()
      },
      ..Default::default()
    };
    fs::write(&config_path, format!("{CONFIG_HEADER}{}", toml::to_string_pretty(&config)?))?;

    // Turns `.glas/` into the versioned meta-repo
    git::init(&glas_dir)?;
    Ok(())
  }

  /// Validate and persist the in-memory config to disk
  pub fn save(&self) -> anyhow::Result<()> {
    validate_config(&self.config)?; // Never persist a broken config

    let path = self.root.join(GLAS_DIR).join(CONFIG_FILE);
    if let Some(parent) = path.parent() {
      fs::create_dir_all(parent)?;
    }
    fs::write(path, format!("{CONFIG_HEADER}{}", toml::to_string_pretty(&self.config)?))?;
    Ok(())
  }
}

/// Load global remotes and return the global secrets path (if the config dir exists)
fn load_global() -> anyhow::Result<(std::collections::HashMap<String, RawRemoteConfig>, Option<PathBuf>)> {
  let Some(home) = std::env::var("HOME").ok().map(PathBuf::from) else {
    return Ok((Default::default(), None));
  };

  let dir = home.join(GLOBAL_CONFIG_SUBDIR);
  let secrets_path = dir.join(GLOBAL_SECRETS_FILE);

  let config_path = dir.join(CONFIG_FILE);
  if !config_path.exists() {
    return Ok((Default::default(), Some(secrets_path)));
  }

  let config: RawWorkspaceConfig = toml::from_str(&fs::read_to_string(&config_path)?)?;
  validate_config(&config)?;

  let global_secrets = if secrets_path.exists() { Some(secrets_path) } else { None };
  Ok((config.remotes, global_secrets))
}

// Write secrets atomically: temp file with 0o600 then rename, so the content is never visible at the target path with wrong permissions
#[cfg(unix)]
fn write_secrets_file(path: &Path, content: &str) -> anyhow::Result<()> {
  use std::os::unix::fs::OpenOptionsExt;

  let tmp = path.with_extension("tmp");
  fs::OpenOptions::new()
    .write(true)
    .create(true)
    .truncate(true)
    .mode(0o600)
    .open(&tmp)?
    .write_all(content.as_bytes())?;
  fs::rename(&tmp, path)?;
  Ok(())
}

#[cfg(not(unix))]
fn write_secrets_file(path: &Path, content: &str) -> anyhow::Result<()> {
  fs::write(path, content)?;
  Ok(())
}
