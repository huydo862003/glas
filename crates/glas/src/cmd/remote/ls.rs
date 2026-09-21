//! glas remote list
//! List all configured remotes and which repos use them

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
    let token_status = if remote.cred.read().is_ok() {
      "token: set"
    } else {
      "token: not set"
    };

    let repo_names: Vec<&str> = config
      .repos
      .iter()
      .filter(|repo| {
        repo
          .primary
          .as_ref()
          .is_some_and(|primary| primary.name == remote.name.as_str())
          || repo
            .push_remotes
            .iter()
            .any(|push| push.name == remote.name.as_str())
      })
      .map(|repo| repo.name.as_str())
      .collect();

    let repos_label = if repo_names.is_empty() {
      String::new()
    } else {
      format!("  repos: {}", repo_names.join(", "))
    };

    logger::print_info(&format!(
      "{}  {}/{}  {token_status}{repos_label}",
      remote.name, remote.url, remote.user
    ));
  }

  Ok(())
}
