//! glas remote remove
//! Remove a remote if no tracked repos reference it

use crate::cli::RemoteRemoveArgs;
use crate::glas::Workspace;
use crate::logger;

pub fn run(args: RemoteRemoveArgs) -> anyhow::Result<()> {
  let mut workspace = Workspace::load()?;

  workspace.remove_remote(&args.name)?;
  workspace.save()?;

  logger::print_ok(&format!("removed remote '{}'", args.name));
  Ok(())
}
