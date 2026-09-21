//! glas pull
//! Pull tracked repos from their primary remote, cloning missing repos

use std::path::Path;

use crate::cli::PullArgs;
use crate::git;
use crate::glas::Workspace;
use crate::logger;
use crate::types::GitRemote;

pub fn run(args: PullArgs) -> anyhow::Result<()> {
  let workspace = Workspace::load()?;
  let config = workspace.config();

  if config.repos.is_empty() {
    logger::print_info("no repos tracked");
    return Ok(());
  }

  let repos: Vec<_> = match &args.repo {
    Some(filter) => {
      let repo = config
        .find_repo(filter)
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
    match &repo.primary {
      None => logger::print_skip(&format!("{}: no primary remote set", repo.name)),
      Some(primary) => {
        if let Err(err) = pull_repo(&repo.name, primary, &repo.path, true) {
          logger::print_error(&format!("{}: {err}", repo.name));
          errors.push(repo.name.to_string());
        }
      }
    }
    progress.inc(1);
  }

  // Pull the meta-repo when syncing all repos
  if args.repo.is_none() {
    progress.set_message("meta-repo");
    if !config.meta.path.exists() {
      logger::print_error(&format!(
        "{}: .glas/ directory missing, run `glas init` to recreate",
        config.meta.name
      ));
      errors.push(config.meta.name.to_string());
    } else {
      match &config.meta.primary {
        None => logger::print_skip(&format!("{}: no primary remote set", config.meta.name)),
        Some(primary) => {
          if let Err(err) = pull_repo(&config.meta.name, primary, &config.meta.path, false) {
            logger::print_error(&format!("{}: {err}", config.meta.name));
            errors.push(config.meta.name.to_string());
          }
        }
      }
    }
    progress.inc(1);
  }
  progress.finish_and_clear();

  if !errors.is_empty() {
    anyhow::bail!("pull failed for: {}", errors.join(", "));
  }

  Ok(())
}

fn pull_repo(
  name: &str,
  primary: &GitRemote,
  path: &Path,
  clone_if_missing: bool,
) -> anyhow::Result<()> {
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
