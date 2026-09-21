//! glas remote add
//! Register a new remote with a base URL and username

use bitflags::bitflags;

use crate::glas::Workspace;
use crate::logger;

bitflags! {
  pub struct Flags: u32 {
    const FORCE = 1 << 0;
  }
}

pub fn run(name: String, url: String, user: String, token: Option<String>, flags: Flags) -> anyhow::Result<()> {
  let mut workspace = Workspace::load()?;

  workspace.add_remote(name.clone(), url, user, flags.contains(Flags::FORCE))?;

  if let Some(token) = token {
    workspace.write_token(&name, token)?;
  }

  workspace.save()?;
  logger::print_ok(&format!("added remote '{name}'"));
  Ok(())
}
