//! CLI definition (clap structs)

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "glas", bin_name = "glas")]
#[command(about = "A git multiplexer - sync repos across multiple providers")]
#[command(version, infer_subcommands = true)]
pub struct Cli {
  #[command(subcommand)]
  pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
  /// Initialize a glas workspace in the current directory
  Init,

  /// Manage remotes
  #[command(subcommand)]
  Remote(RemoteCommand),

  /// Manage tracked repos
  #[command(subcommand)]
  Repo(RepoCommand),

  /// Push all tracked repos to all their remotes
  Push,

  /// Pull all tracked repos from their primary remote
  Pull,
}

#[derive(Subcommand)]
pub enum RemoteCommand {
  /// Add a remote
  Add {
    /// Remote name (e.g. github, gitlab, bitbucket)
    name: String,
    /// Base URL (e.g. https://github.com)
    url: String,
    /// Username or organization on this remote
    user: String,
    /// Auth token (optional, stored in secrets.toml)
    #[arg(long)]
    token: Option<String>,
    /// Overwrite if the remote already exists
    #[arg(long)]
    force: bool,
  },

  /// List configured remotes
  Ls,

  /// Remove a remote
  Rm {
    /// Remote name to remove
    name: String,
  },
}

#[derive(Subcommand)]
pub enum RepoCommand {
  /// Track a repo (defaults to current directory)
  Add {
    /// Path to a local git repo outside the workspace
    path: Option<String>,
    /// Overwrite if the repo is already tracked
    #[arg(long)]
    force: bool,
  },

  /// Stop tracking the current directory as a repo
  Rm,

  /// List all tracked repos
  Ls,

  /// Show git status of all tracked repos
  Status,

  /// Manage remotes for a tracked repo
  #[command(subcommand)]
  Remote(RepoRemoteCommand),

  /// Set repo properties
  #[command(subcommand)]
  Set(RepoSetCommand),
}

#[derive(Subcommand)]
pub enum RepoRemoteCommand {
  /// Add a remote to the current repo
  Add {
    /// Remote name
    remote: String,
    /// Custom repo name on this remote
    #[arg(long)]
    repo: Option<String>,
    /// Custom user on this remote
    #[arg(long)]
    user: Option<String>,
    /// Set as primary source
    #[arg(long)]
    primary: bool,
    /// Overwrite if the remote is already set on this repo
    #[arg(long)]
    force: bool,
  },

  /// Remove a remote from the current repo
  Rm {
    /// Remote name
    remote: String,
  },

  /// List remotes for the current repo
  Ls,
}

#[derive(Subcommand)]
pub enum RepoSetCommand {
  /// Set the primary remote for the current repo
  Primary {
    /// Remote name
    remote: String,
    /// Custom repo name on this remote
    #[arg(long)]
    repo: Option<String>,
    /// Custom user on this remote
    #[arg(long)]
    user: Option<String>,
  },
}
