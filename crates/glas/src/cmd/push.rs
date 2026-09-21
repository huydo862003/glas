//! glas push
//! Push all tracked repos and the meta-repo to their remotes

use std::path::Path;

use crate::git;
use crate::glas::Workspace;
use crate::logger;
use crate::types::GitRemote;

pub fn run() -> anyhow::Result<()> {
  let workspace = Workspace::load()?;
  let config = workspace.config();

  if config.repos.is_empty() {
    logger::print_info("no repos tracked");
    return Ok(());
  }

  let total = config.repos.len() as u64 + 1;
  let progress = logger::create_progress_bar(total);

  // Push each tracked repo to its remotes
  for repo in &config.repos {
    progress.set_message(repo.name.to_string());

    if !repo.path.exists() {
      logger::print_skip(&format!("{}: not cloned locally", repo.name));
      progress.inc(1);
      continue;
    }

    push_remotes(&repo.name, &repo.push_remotes, &repo.path)?;
    progress.inc(1);
  }

  // Push the meta-repo itself
  progress.set_message("meta-repo");
  push_remotes(
    &config.meta.name,
    &config.meta.push_remotes,
    &config.meta.path,
  )?;
  progress.inc(1);
  progress.finish_and_clear();

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
