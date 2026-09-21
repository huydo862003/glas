//! glas repo remove
//! Stop tracking a repo

use crate::glas::Workspace;
use crate::logger;

pub fn run() -> anyhow::Result<()> {
  let mut workspace = Workspace::load()?;
  let name = workspace.get_current_repo_name()?;

  workspace.remove_repo(&name)?;
  workspace.save()?;

  logger::print_ok(&format!("untracked '{name}'"));
  Ok(())
}
