//! glas status
//! Show workspace overview: remotes, repos, and their state

use crate::git;
use crate::glas::Workspace;
use crate::logger;

pub fn run() -> anyhow::Result<()> {
  let workspace = Workspace::load()?;
  let config = workspace.config();

  println!("workspace: {}", workspace.root().display());
  println!();

  // Remotes
  if config.remotes.is_empty() {
    println!("remotes: (none)");
  } else {
    println!("remotes:");
    for remote in &config.remotes {
      let token_status = if remote.cred.read().is_ok() { "token: set" } else { "token: not set" };
      println!("  {}  {}/{}  {}", remote.name, remote.url, remote.user, token_status);
    }
  }
  println!();

  // Repos
  if config.repos.is_empty() {
    println!("repos: (none)");
  } else {
    println!("repos:");
    for repo in &config.repos {
      let primary_label = repo.primary.as_ref()
        .map(|primary| primary.name.to_string())
        .unwrap_or_else(|| "(no primary)".to_string());

      if !repo.path.exists() {
        logger::print_skip(&format!("  {}  primary={}  not cloned", repo.name, primary_label));
        continue;
      }

      let dirty_label = match git::is_dirty(&repo.path) {
        Ok(true) => "dirty",
        Ok(false) => "clean",
        Err(_) => "unknown",
      };
      println!("  {}  primary={}  {}", repo.name, primary_label, dirty_label);
    }
  }

  Ok(())
}
