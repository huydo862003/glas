//! glas repo remote ls
//! List remotes configured for the current repo

use crate::glas::Workspace;
use crate::logger;

pub fn run() -> anyhow::Result<()> {
  let workspace = Workspace::load()?;
  let name = workspace.get_current_repo_name()?;
  let config = workspace.config();

  let repo = config
    .find_repo(&name)
    .ok_or_else(|| anyhow::anyhow!("repo '{name}' is not tracked"))?;

  if let Some(primary) = &repo.primary {
    logger::print_info(&format!("primary: {}  {}  {}", primary.name, primary.user, primary.url));
  } else {
    logger::print_info("primary: (not set)");
  }

  if repo.push_remotes.is_empty() {
    logger::print_info("remotes: (none)");
  } else {
    for remote in &repo.push_remotes {
      logger::print_info(&format!("  {}  {}  {}", remote.name, remote.user, remote.url));
    }
  }

  Ok(())
}
