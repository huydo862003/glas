//! glas repo primary
//! Set the primary (pull) remote for the current repo

use crate::cli::RepoPrimaryArgs;
use crate::glas::Workspace;
use crate::logger;

pub fn run(args: RepoPrimaryArgs) -> anyhow::Result<()> {
  let mut workspace = Workspace::load()?;
  let name = workspace.get_current_repo_name()?;

  workspace.set_repo_primary(&name, args.remote.clone(), args.repo, args.user)?;
  workspace.save()?;

  logger::print_ok(&format!("{name}: set primary to '{}'", args.remote));
  Ok(())
}
