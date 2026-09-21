mod cli;
mod cmd;
mod git;
mod glas;
mod logger;
mod types;

use clap::Parser;

use cli::{Cli, Command, RemoteCommand, RepoCommand, RepoRemoteCommand, RepoSetCommand};

fn main() -> anyhow::Result<()> {
  let cli = Cli::parse();

  match cli.command {
    Command::Init => cmd::init::run()?,
    Command::Remote(cmd) => match cmd {
      RemoteCommand::Add {
        name,
        url,
        user,
        token,
        force,
      } => {
        let mut flags = cmd::remote::add::Flags::empty();
        if force {
          flags |= cmd::remote::add::Flags::FORCE;
        }
        cmd::remote::add::run(name, url, user, token, flags)?
      }
      RemoteCommand::Ls => cmd::remote::ls::run()?,
      RemoteCommand::Rm { name } => cmd::remote::rm::run(name)?,
    },
    Command::Repo(cmd) => match cmd {
      RepoCommand::Add { path, force } => {
        let mut flags = cmd::repo::add::Flags::empty();
        if force {
          flags |= cmd::repo::add::Flags::FORCE;
        }
        cmd::repo::add::run(path, flags)?
      }
      RepoCommand::Rm => cmd::repo::rm::run()?,
      RepoCommand::Ls => cmd::repo::ls::run()?,
      RepoCommand::Status => cmd::repo::status::run()?,
      RepoCommand::Remote(cmd) => match cmd {
        RepoRemoteCommand::Add {
          remote,
          repo,
          user,
          primary,
          force,
        } => {
          let mut flags = cmd::repo::remote::add::Flags::empty();
          if force {
            flags |= cmd::repo::remote::add::Flags::FORCE;
          }
          cmd::repo::remote::add::run(remote, repo, user, primary, flags)?
        }
        RepoRemoteCommand::Rm { remote } => cmd::repo::remote::rm::run(remote)?,
        RepoRemoteCommand::Ls => cmd::repo::remote::ls::run()?,
      },
      RepoCommand::Set(cmd) => match cmd {
        RepoSetCommand::Primary { remote, repo, user } => {
          cmd::repo::set::primary::run(remote, repo, user)?
        }
      },
    },
    Command::Push => cmd::push::run()?,
    Command::Pull => cmd::pull::run()?,
  }

  Ok(())
}
