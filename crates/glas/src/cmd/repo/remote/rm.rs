//! glas repo remote rm
//! Remove a remote from the current repo

use crate::glas::Workspace;
use crate::logger;

pub fn run(remote: String) -> anyhow::Result<()> {
  let mut workspace = Workspace::load()?;
  let name = workspace.get_current_repo_name()?;

  workspace.remove_repo_remote(&name, &remote)?;
  workspace.save()?;

  logger::print_ok(&format!("{name}: removed remote '{remote}'"));
  Ok(())
}
