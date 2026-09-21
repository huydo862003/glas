//! glas push
//! Push tracked repos and the meta-repo to their remotes

use std::path::Path;

use crate::cli::PushArgs;
use crate::git;
use crate::glas::Workspace;
use crate::logger;
use crate::types::GitRemote;

pub fn run(args: PushArgs) -> anyhow::Result<()> {
  let workspace = Workspace::load()?;
  let config = workspace.config();

  if config.repos.is_empty() {
    logger::print_info("no repos tracked");
    return Ok(());
  }

  let repos: Vec<_> = match &args.repo {
    Some(filter) => {
      let repo = config.find_repo(filter)
        .ok_or_else(|| anyhow::anyhow!("repo '{filter}' is not tracked"))?;
      vec![repo]
    }
    None => config.repos.iter().collect(),
  };

  let total = repos.len() as u64 + if args.repo.is_none() { 1 } else { 0 };
  let progress = logger::create_progress_bar(total);
  let mut errors: Vec<String> = Vec::new();

  for repo in &repos {
    progress.set_message(repo.name.to_string());

    if !repo.path.exists() {
      logger::print_skip(&format!("{}: not cloned locally", repo.name));
      progress.inc(1);
      continue;
    }

    if repo.push_remotes.is_empty() {
      logger::print_skip(&format!("{}: no remotes configured", repo.name));
      progress.inc(1);
      continue;
    }

    if let Err(err) = push_remotes(&repo.name, &repo.push_remotes, &repo.path) {
      logger::print_error(&format!("{}: {err}", repo.name));
      errors.push(repo.name.to_string());
    }
    progress.inc(1);
  }

  // Push the meta-repo when syncing all repos
  if args.repo.is_none() {
    progress.set_message("meta-repo");
    if let Err(err) = push_remotes(&config.meta.name, &config.meta.push_remotes, &config.meta.path) {
      logger::print_error(&format!("{}: {err}", config.meta.name));
      errors.push(config.meta.name.to_string());
    }
    progress.inc(1);
  }
  progress.finish_and_clear();

  if !errors.is_empty() {
    anyhow::bail!("push failed for: {}", errors.join(", "));
  }

  Ok(())
}

fn push_remotes(name: &str, remotes: &[GitRemote], path: &Path) -> anyhow::Result<()> {
  for remote in remotes {
    remote.cred.ensure_repo_exists(&remote.user, name, true)?;
    git::check_remote_exists(path, remote.name.as_str(), &remote.url)?;
    git::push(path, remote.name.as_str(), &remote.cred)?;
    logger::print_ok(&format!("{name} to {}: push", remote.name));
  }
  Ok(())
}
