//! glas remote list
//! List all configured remotes

use crate::glas::Workspace;
use crate::logger;

pub fn run() -> anyhow::Result<()> {
  let workspace = Workspace::load()?;
  let config = workspace.config();

  if config.remotes.is_empty() {
    logger::print_info("no remotes configured");
    return Ok(());
  }

  for remote in &config.remotes {
    let token_status = if remote.cred.read().is_ok() { "token: set" } else { "token: not set" };
    logger::print_info(&format!("{}  {}  {}  {token_status}", remote.name, remote.url, remote.user));
  }

  Ok(())
}
