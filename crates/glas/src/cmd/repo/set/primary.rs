//! glas repo set primary
//! Set the primary remote for the current repo

use crate::glas::Workspace;
use crate::logger;

pub fn run(
  remote: String,
  repo_name_override: Option<String>,
  user_override: Option<String>,
) -> anyhow::Result<()> {
  let mut workspace = Workspace::load()?;
  let name = workspace.get_current_repo_name()?;

  workspace.set_repo_primary(&name, remote.clone(), repo_name_override, user_override)?;
  workspace.save()?;

  logger::print_ok(&format!("{name}: set primary to '{remote}'"));
  Ok(())
}
