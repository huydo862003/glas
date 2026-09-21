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
  Init(InitArgs),

  /// Manage shared remote definitions
  #[command(subcommand)]
  Remote(RemoteCommand),

  /// Manage tracked repos
  #[command(subcommand)]
  Repo(RepoCommand),

  /// Push tracked repos to their remotes
  Push(PushArgs),

  /// Pull tracked repos from their primary remote
  Pull(PullArgs),

  /// Show workspace overview
  Status,
}

#[derive(clap::Args)]
pub struct InitArgs {
  /// Name of the meta-repo on remotes
  #[arg(long, default_value = "glas")]
  pub meta_repo: String,
}

#[derive(clap::Args)]
pub struct PushArgs {
  /// Push only this repo (default: all)
  pub repo: Option<String>,
}

#[derive(clap::Args)]
pub struct PullArgs {
  /// Pull only this repo (default: all)
  pub repo: Option<String>,
}

#[derive(Subcommand)]
pub enum RemoteCommand {
  /// Add a remote
  Add(RemoteAddArgs),

  /// List configured remotes
  List,

  /// Remove a remote
  Remove(RemoteRemoveArgs),
}

#[derive(clap::Args)]
pub struct RemoteAddArgs {
  /// Remote name (e.g. github, gitlab, bitbucket)
  pub name: String,
  /// URL with user/org (e.g. https://github.com/myorg)
  pub url: String,
  /// Auth token, stored in ~/.config/glas/secrets.toml
  #[arg(long)]
  pub token: Option<String>,
  /// Overwrite if the remote already exists
  #[arg(long)]
  pub force: bool,
}

#[derive(clap::Args)]
pub struct RemoteRemoveArgs {
  /// Remote name to remove
  pub name: String,
}

#[derive(Subcommand)]
pub enum RepoCommand {
  /// Track a repo (defaults to current directory)
  Add(RepoAddArgs),

  /// Stop tracking a repo (defaults to current directory)
  Remove(RepoRemoveArgs),

  /// List all tracked repos
  List,

  /// Show git status of all tracked repos
  Status,

  /// Set the primary (pull) remote for a repo
  Primary(RepoPrimaryArgs),
}

#[derive(clap::Args)]
pub struct RepoAddArgs {
  /// Path to a local git repo outside the workspace
  pub path: Option<String>,
  /// Overwrite if the repo is already tracked
  #[arg(long)]
  pub force: bool,
}

#[derive(clap::Args)]
pub struct RepoRemoveArgs {
  /// Repo name (default: current directory name)
  pub name: Option<String>,
}

#[derive(clap::Args)]
pub struct RepoPrimaryArgs {
  /// Remote name
  pub remote: String,
  /// Override repo name on this remote (when it differs from local name)
  #[arg(long)]
  pub repo: Option<String>,
  /// Override user/org on this remote (when it differs from remote default)
  #[arg(long)]
  pub user: Option<String>,
}
