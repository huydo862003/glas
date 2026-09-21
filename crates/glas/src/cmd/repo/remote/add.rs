//! glas repo remote add
//! Add a remote to the current repo, optionally setting it as primary

use bitflags::bitflags;

use crate::glas::Workspace;
use crate::logger;

bitflags! {
  pub struct Flags: u32 {
    const FORCE = 1 << 0;
  }
}

pub fn run(
  remote: String,
  repo_name_override: Option<String>,
  user_override: Option<String>,
  primary: bool,
  flags: Flags,
) -> anyhow::Result<()> {
  let mut workspace = Workspace::load()?;
  let name = workspace.get_current_repo_name()?;
  let force = flags.contains(Flags::FORCE);

  let message = if primary {
    workspace.set_repo_primary(&name, remote.clone(), repo_name_override, user_override)?;
    format!("{name}: set primary to '{remote}'")
  } else {
    workspace.add_repo_remote(&name, remote.clone(), repo_name_override, user_override, force)?;
    format!("{name}: added remote '{remote}'")
  };

  workspace.save()?;
  logger::print_ok(&message);
  Ok(())
}
