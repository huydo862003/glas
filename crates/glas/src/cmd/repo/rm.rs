//! glas repo remove
//! Stop tracking a repo

use crate::cli::RepoRemoveArgs;
use crate::glas::Workspace;
use crate::logger;

pub fn run(args: RepoRemoveArgs) -> anyhow::Result<()> {
  let mut workspace = Workspace::load()?;
  let name = match args.name {
    Some(name) => name,
    None => workspace.get_current_repo_name()?,
  };

  workspace.remove_repo(&name)?;
  workspace.save()?;

  logger::print_ok(&format!("untracked '{name}'"));
  Ok(())
}
