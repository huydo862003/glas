//! glas remote add
//! Register a new shared remote definition

use crate::cli::RemoteAddArgs;
use crate::glas::Workspace;
use crate::logger;

pub fn run(args: RemoteAddArgs) -> anyhow::Result<()> {
  let (base_url, user) = parse_remote_url(&args.url)?;
  let mut workspace = Workspace::load()?;

  if args.global {
    if !args.force && workspace.config().remotes.iter().any(|remote| remote.name == args.name.as_str()) {
      anyhow::bail!("remote '{}' already exists", args.name);
    }
    workspace.write_global_remote(&args.name, base_url, user, args.token)?;
    logger::print_ok(&format!("added global remote '{}'", args.name));
    return Ok(());
  }

  workspace.add_remote(args.name.clone(), base_url, user, args.force)?;

  if let Some(token) = args.token {
    workspace.write_token(&args.name, token)?;
  }

  workspace.save()?;
  logger::print_ok(&format!("added remote '{}'", args.name));
  Ok(())
}

/// Parse "https://github.com/myorg" into ("https://github.com", "myorg")
fn parse_remote_url(url: &str) -> anyhow::Result<(String, String)> {
  let trimmed = url.trim_end_matches('/');
  let last_slash = trimmed.rfind('/').ok_or_else(|| {
    anyhow::anyhow!("invalid remote URL '{url}': expected https://host/user")
  })?;

  let base = &trimmed[..last_slash];
  let user = &trimmed[last_slash + 1..];

  if user.is_empty() || base.is_empty() {
    anyhow::bail!("invalid remote URL '{url}': expected https://host/user");
  }

  if !base.contains("://") {
    anyhow::bail!("invalid remote URL '{url}': expected https://host/user");
  }

  Ok((base.to_string(), user.to_string()))
}
