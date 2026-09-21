//! glas repo status
//! Show clean/dirty status of all tracked repos

use crate::git;
use crate::glas::Workspace;
use crate::logger;

pub fn run() -> anyhow::Result<()> {
  let workspace = Workspace::load()?;
  let config = workspace.config();

  if config.repos.is_empty() {
    logger::print_info("no repos tracked");
    return Ok(());
  }

  for repo in &config.repos {
    if !repo.path.exists() {
      logger::print_skip(&format!("{}  not cloned locally", repo.name));
      continue;
    }

    let label = if git::is_dirty(&repo.path)? {
      "dirty"
    } else {
      "clean"
    };
    logger::print_info(&format!("{}  {label}", repo.name));
  }

  Ok(())
}
