//! glas pull
//! Pull all tracked repos from their primary remote, cloning missing repos

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

  for repo in &config.repos {
    progress.set_message(repo.name.to_string());
    match &repo.primary {
      None => logger::print_skip(&format!("{}: no primary remote set", repo.name)),
      Some(primary) => pull_repo(&repo.name, primary, &repo.path, true)?,
    }
    progress.inc(1);
  }

  // Pull the meta-repo itself; never clone it, it must already exist
  progress.set_message("meta-repo");
  match &config.meta.primary {
    None => logger::print_skip(&format!("{}: no primary remote set", config.meta.name)),
    Some(primary) => pull_repo(&config.meta.name, primary, &config.meta.path, false)?,
  }
  progress.inc(1);
  progress.finish_and_clear();

  Ok(())
}

fn pull_repo(name: &str, primary: &GitRemote, path: &Path, clone_if_missing: bool) -> anyhow::Result<()> {
  if clone_if_missing && !path.exists() {
    logger::print_info(&format!("{name}: cloning from {}...", primary.name));
    git::clone(&primary.url, path, &primary.cred)?;
    return Ok(());
  }

  git::check_remote_exists(path, primary.name.as_str(), &primary.url)?;
  git::pull(path, primary.name.as_str(), &primary.cred)?;
  logger::print_ok(&format!("{name} from {}: pull", primary.name));
  Ok(())
}
