//! Git operations used across commands

use std::path::Path;

use git2::build::CheckoutBuilder;
use secrecy::ExposeSecret;

use crate::glas::cred::Credential;

/// Initialize a new git repository
pub fn init(path: &Path) -> anyhow::Result<()> {
  git2::Repository::init(path)
    .map_err(|err| anyhow::anyhow!("git init failed at {}: {err}", path.display()))?;
  Ok(())
}

/// Clone a repo from `url` into `dest`
pub fn clone(url: &str, dest: &Path, cred: &Credential) -> anyhow::Result<()> {
  let mut builder = git2::build::RepoBuilder::new();
  builder.fetch_options(fetch_options(cred));
  builder
    .clone(url, dest)
    .map_err(|err| anyhow::anyhow!("clone failed for {url}: {err}"))?;
  Ok(())
}

/// Push all local branches to a named remote
pub fn push(repo_path: &Path, remote_name: &str, cred: &Credential) -> anyhow::Result<()> {
  let repo = open(repo_path)?;
  let mut remote = repo
    .find_remote(remote_name)
    .map_err(|err| anyhow::anyhow!("remote '{remote_name}' not found: {err}"))?;
  let mut push_opts = git2::PushOptions::new();
  push_opts.remote_callbacks(remote_callbacks(cred));
  remote
    .push(&["+refs/heads/*:refs/heads/*"], Some(&mut push_opts))
    .map_err(|err| anyhow::anyhow!("push failed to '{remote_name}': {err}"))?;
  Ok(())
}

/// Fetch from a named remote and fast-forward the current branch
pub fn pull(repo_path: &Path, remote_name: &str, cred: &Credential) -> anyhow::Result<()> {
  let repo = open(repo_path)?;

  // Fetch
  let mut remote = repo
    .find_remote(remote_name)
    .map_err(|err| anyhow::anyhow!("remote '{remote_name}' not found: {err}"))?;
  remote
    .fetch(&[] as &[&str], Some(&mut fetch_options(cred)), None)
    .map_err(|err| anyhow::anyhow!("fetch failed from '{remote_name}': {err}"))?;
  drop(remote);

  // Resolve what was fetched
  let fetch_head = repo
    .find_reference("FETCH_HEAD")
    .map_err(|err| anyhow::anyhow!("FETCH_HEAD not found after fetch: {err}"))?;
  let fetch_commit = repo
    .reference_to_annotated_commit(&fetch_head)
    .map_err(|err| anyhow::anyhow!("could not resolve fetch commit: {err}"))?;

  let (analysis, _) = repo
    .merge_analysis(&[&fetch_commit])
    .map_err(|err| anyhow::anyhow!("merge analysis failed: {err}"))?;

  if analysis.is_up_to_date() {
    return Ok(());
  }
  if !analysis.is_fast_forward() {
    anyhow::bail!("branches have diverged; merge manually");
  }

  // Fast-forward the current branch
  let head = repo.head().map_err(|err| anyhow::anyhow!("no HEAD: {err}"))?;
  let branch_name = head
    .shorthand()
    .ok_or_else(|| anyhow::anyhow!("HEAD is not a named branch"))?
    .to_string();
  let refname = format!("refs/heads/{branch_name}");
  let mut branch_ref = repo
    .find_reference(&refname)
    .map_err(|err| anyhow::anyhow!("could not find branch ref '{refname}': {err}"))?;
  branch_ref
    .set_target(fetch_commit.id(), "fast-forward")
    .map_err(|err| anyhow::anyhow!("fast-forward failed: {err}"))?;
  repo
    .set_head(&refname)
    .map_err(|err| anyhow::anyhow!("set HEAD failed: {err}"))?;
  repo
    .checkout_head(Some(CheckoutBuilder::default().force()))
    .map_err(|err| anyhow::anyhow!("checkout failed: {err}"))?;

  Ok(())
}

/// Return true if the repo has any uncommitted changes or untracked files
pub fn is_dirty(repo_path: &Path) -> anyhow::Result<bool> {
  let repo = open(repo_path)?;
  let statuses = repo
    .statuses(None)
    .map_err(|err| anyhow::anyhow!("status failed: {err}"))?;
  Ok(!statuses.is_empty())
}

/// Ensure a named remote exists in the repo config pointing at the given URL
pub fn check_remote_exists(repo_path: &Path, name: &str, url: &str) -> anyhow::Result<()> {
  let repo = open(repo_path)?;
  match repo.find_remote(name) {
    Ok(_) => repo
      .remote_set_url(name, url)
      .map_err(|err| anyhow::anyhow!("could not update remote '{name}': {err}"))?,
    Err(ref err) if err.code() == git2::ErrorCode::NotFound => {
      repo
        .remote(name, url)
        .map_err(|err| anyhow::anyhow!("could not add remote '{name}': {err}"))?;
    }
    Err(err) => anyhow::bail!("could not check remote '{name}': {err}"),
  }
  Ok(())
}

/// Check if a path is a git repository
pub fn is_dir_git_repo(path: &Path) -> bool {
  git2::Repository::open(path).is_ok()
}

fn open(path: &Path) -> anyhow::Result<git2::Repository> {
  git2::Repository::open(path)
    .map_err(|err| anyhow::anyhow!("not a git repository at {}: {err}", path.display()))
}

/// Credential callback chain: our token -> SSH agent -> system credential helper (GIT_ASKPASS etc.)
fn remote_callbacks(cred: &Credential) -> git2::RemoteCallbacks<'static> {
  let token: Option<String> = cred.read().ok().map(|s| s.expose_secret().to_string());
  let mut callbacks = git2::RemoteCallbacks::new();
  callbacks.credentials(move |url, username, allowed| {
    if allowed.contains(git2::CredentialType::USER_PASS_PLAINTEXT)
      && let Some(ref tok) = token
    {
      return git2::Cred::userpass_plaintext("", tok);
    }
    if allowed.contains(git2::CredentialType::SSH_KEY)
      && let Ok(cred) = git2::Cred::ssh_key_from_agent(username.unwrap_or("git"))
    {
      return Ok(cred);
    }
    // Fall back to the system credential helper (respects GIT_ASKPASS, keychain, etc.)
    git2::Cred::credential_helper(&git2::Config::open_default()?, url, username)
  });
  callbacks
}

fn fetch_options(cred: &Credential) -> git2::FetchOptions<'static> {
  let mut opts = git2::FetchOptions::new();
  opts.remote_callbacks(remote_callbacks(cred));
  opts
}
