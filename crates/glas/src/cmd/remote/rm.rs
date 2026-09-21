//! glas remote rm
//! Remove a remote if no tracked repos reference it

use crate::glas::Workspace;
use crate::logger;

pub fn run(name: String) -> anyhow::Result<()> {
  let mut workspace = Workspace::load()?;

  workspace.remove_remote(&name)?;
  workspace.save()?;

  logger::print_ok(&format!("removed remote '{name}'"));
  Ok(())
}
