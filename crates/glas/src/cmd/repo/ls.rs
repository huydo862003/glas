//! glas repo list
//! List all tracked repos and their remotes

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
    let primary_label = repo
      .primary
      .as_ref()
      .map(|primary| primary.name.as_str())
      .unwrap_or("(none)");
    let remote_names: Vec<&str> = repo
      .push_remotes
      .iter()
      .map(|remote| remote.name.as_str())
      .collect();
    logger::print_info(&format!(
      "{}  primary={primary_label} remotes=[{}]",
      repo.name,
      remote_names.join(", ")
    ));
  }

  Ok(())
}
