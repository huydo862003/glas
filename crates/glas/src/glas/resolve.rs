use std::collections::HashMap;
use std::path::Path;

use crate::glas::constants::{GLAS_DIR, SECRETS_PATH};
use crate::glas::cred::Credential;
use crate::glas::providers::{self, GitProvider};
use crate::glas::types::{RawGitRemote, RawRemoteConfig, RawWorkspaceConfig};
use crate::types::{
  GitRemote, GitRemoteProvider, GitRepository, RemoteConfig, WorkspaceConfig, WorkspaceMeta,
};

pub fn resolve_config(
  config: &RawWorkspaceConfig,
  workspace_root: &Path,
  global_secrets_path: Option<&Path>,
) -> WorkspaceConfig {
  let meta_name = config.meta.repo.as_deref().unwrap_or("glas");
  let secrets_path = workspace_root.join(SECRETS_PATH);

  let remotes = resolve_global_remotes(&config.remotes, &secrets_path, global_secrets_path);

  let meta = WorkspaceMeta {
    name: meta_name
      .parse()
      .expect("validate_config guarantees valid repo name"),
    path: workspace_root.join(GLAS_DIR),
    primary: resolve_primary(
      &config.remotes,
      &config.meta.primary,
      meta_name,
      &secrets_path,
      global_secrets_path,
    ),
    push_remotes: resolve_push_remotes(
      &config.remotes,
      &config.meta.remotes,
      meta_name,
      &secrets_path,
      global_secrets_path,
    ),
  };

  let mut repos: Vec<GitRepository> = config
    .repo
    .iter()
    .map(|(name, repo)| GitRepository {
      name: name
        .parse()
        .expect("validate_config guarantees valid repo name"),
      path: workspace_root.join(name),
      primary: resolve_primary(
        &config.remotes,
        &repo.primary,
        name,
        &secrets_path,
        global_secrets_path,
      ),
      push_remotes: resolve_push_remotes(
        &config.remotes,
        &repo.remotes,
        name,
        &secrets_path,
        global_secrets_path,
      ),
    })
    .collect();
  repos.sort_by(|left, right| left.name.as_str().cmp(right.name.as_str()));

  WorkspaceConfig {
    meta,
    remotes,
    repos,
  }
}

fn make_cred(
  name: &str,
  secrets_path: &Path,
  global_secrets_path: Option<&Path>,
  provider: Option<Box<dyn GitProvider>>,
) -> Credential {
  let env_var = format!("GLAS_TOKEN_{}", name.to_uppercase().replace('-', "_"));
  Credential::new(
    provider,
    Some(secrets_path.to_path_buf()),
    global_secrets_path.map(|p| p.to_path_buf()),
    name.to_string(),
    Some(env_var),
  )
}

fn build_provider(provider: &GitRemoteProvider, base_url: &str) -> Option<Box<dyn GitProvider>> {
  let host = extract_host(base_url);
  Some(match provider {
    GitRemoteProvider::GitHub => Box::new(providers::GitHub { host }) as Box<dyn GitProvider>,
    GitRemoteProvider::GitLab => Box::new(providers::GitLab { host }),
    GitRemoteProvider::Gitea | GitRemoteProvider::Forgejo => Box::new(providers::Gitea {
      base_url: base_url.to_string(),
    }),
    GitRemoteProvider::Bitbucket => Box::new(providers::Bitbucket),
    GitRemoteProvider::Other(_) => return None,
  })
}

fn extract_host(url: &str) -> String {
  url
    .trim_start_matches("https://")
    .trim_start_matches("http://")
    .split('/')
    .next()
    .unwrap_or(url)
    .to_string()
}

fn resolve_global_remotes(
  global_remotes: &HashMap<String, RawRemoteConfig>,
  secrets_path: &Path,
  global_secrets_path: Option<&Path>,
) -> Vec<RemoteConfig> {
  let mut remotes: Vec<RemoteConfig> = global_remotes
    .iter()
    .map(|(name, raw)| {
      let provider: GitRemoteProvider = name
        .parse()
        .expect("validate_config guarantees valid provider name");
      let cli_provider = build_provider(&provider, &raw.url);
      RemoteConfig {
        name: provider,
        url: raw
          .url
          .parse()
          .expect("validate_config guarantees valid URL"),
        user: raw
          .user
          .parse()
          .expect("validate_config guarantees valid username"),
        cred: make_cred(name, secrets_path, global_secrets_path, cli_provider),
      }
    })
    .collect();
  remotes.sort_by(|left, right| left.name.as_str().cmp(right.name.as_str()));
  remotes
}

fn build_git_remote(
  name: &str,
  full_url: &str,
  user: &str,
  base_url: &str,
  secrets_path: &Path,
  global_secrets_path: Option<&Path>,
) -> GitRemote {
  let provider: GitRemoteProvider = name
    .parse()
    .expect("validate_config guarantees valid provider name");
  let cli_provider = build_provider(&provider, base_url);
  GitRemote {
    name: provider,
    url: full_url.parse().expect("URL built from validated parts"),
    user: user
      .parse()
      .expect("validate_config guarantees valid username"),
    cred: make_cred(name, secrets_path, global_secrets_path, cli_provider),
  }
}

fn resolve_primary(
  global_remotes: &HashMap<String, RawRemoteConfig>,
  primary: &Option<RawGitRemote>,
  repo_name: &str,
  secrets_path: &Path,
  global_secrets_path: Option<&Path>,
) -> Option<GitRemote> {
  primary.as_ref().map(|raw| {
    let global = global_remotes
      .get(&raw.name)
      .expect("validate_config guarantees remote exists");
    let user = raw.user.as_deref().unwrap_or(&global.user);
    let repo = raw.repo.as_deref().unwrap_or(repo_name);
    let full_url = format!("{}/{}/{}.git", global.url, user, repo);
    build_git_remote(
      &raw.name,
      &full_url,
      user,
      &global.url,
      secrets_path,
      global_secrets_path,
    )
  })
}

fn resolve_push_remotes(
  global_remotes: &HashMap<String, RawRemoteConfig>,
  remotes: &[RawGitRemote],
  repo_name: &str,
  secrets_path: &Path,
  global_secrets_path: Option<&Path>,
) -> Vec<GitRemote> {
  if !remotes.is_empty() {
    return remotes
      .iter()
      .map(|raw| {
        let global = global_remotes
          .get(&raw.name)
          .expect("validate_config guarantees remote exists");
        let user = raw.user.as_deref().unwrap_or(&global.user);
        let repo = raw.repo.as_deref().unwrap_or(repo_name);
        let full_url = format!("{}/{}/{}.git", global.url, user, repo);
        build_git_remote(
          &raw.name,
          &full_url,
          user,
          &global.url,
          secrets_path,
          global_secrets_path,
        )
      })
      .collect();
  }
  // Fall back to all global remotes
  let mut result: Vec<GitRemote> = global_remotes
    .iter()
    .map(|(name, global)| {
      let full_url = format!("{}/{}/{}.git", global.url, global.user, repo_name);
      build_git_remote(
        name,
        &full_url,
        &global.user,
        &global.url,
        secrets_path,
        global_secrets_path,
      )
    })
    .collect();
  result.sort_by(|left, right| left.name.as_str().cmp(right.name.as_str()));
  result
}
