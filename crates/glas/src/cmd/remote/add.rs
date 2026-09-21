//! glas remote add
//! Register a new remote with a base URL and username

use crate::cli::RemoteAddArgs;
use crate::glas::Workspace;
use crate::logger;

pub fn run(args: RemoteAddArgs) -> anyhow::Result<()> {
  let (base_url, user) = parse_remote_url(&args.url)?;
  let mut workspace = Workspace::load()?;

  workspace.add_remote(args.name.clone(), base_url, user, args.force)?;

  if let Some(token) = args.token {
    workspace.write_global_token(&args.name, token)?;
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

  if !base.starts_with("https://") && !base.starts_with("http://") {
    anyhow::bail!("invalid remote URL '{url}': expected https://host/user");
  }

  Ok((base.to_string(), user.to_string()))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse_https_url() {
    let (base, user) = parse_remote_url("https://github.com/myorg").unwrap();
    assert_eq!(base, "https://github.com");
    assert_eq!(user, "myorg");
  }

  #[test]
  fn parse_http_url() {
    let (base, user) = parse_remote_url("http://gitlab.local/team").unwrap();
    assert_eq!(base, "http://gitlab.local");
    assert_eq!(user, "team");
  }

  #[test]
  fn parse_url_with_trailing_slash() {
    let (base, user) = parse_remote_url("https://github.com/myorg/").unwrap();
    assert_eq!(base, "https://github.com");
    assert_eq!(user, "myorg");
  }

  #[test]
  fn reject_ftp_scheme() {
    assert!(parse_remote_url("ftp://github.com/myorg").is_err());
  }

  #[test]
  fn reject_no_scheme() {
    assert!(parse_remote_url("github.com/myorg").is_err());
  }

  #[test]
  fn reject_no_user() {
    assert!(parse_remote_url("https://github.com").is_err());
  }

  #[test]
  fn reject_empty() {
    assert!(parse_remote_url("").is_err());
  }
}
