mod cli;
mod cmd;
mod git;
mod glas;
mod logger;
mod types;

use clap::Parser;

use cli::{Cli, Command, RemoteCommand, RepoCommand};

fn main() -> anyhow::Result<()> {
  let cli = Cli::parse();

  match cli.command {
    Command::Init(args) => cmd::init::run(args)?,
    Command::Remote(cmd) => match cmd {
      RemoteCommand::Add(args) => cmd::remote::add::run(args)?,
      RemoteCommand::List => cmd::remote::ls::run()?,
      RemoteCommand::Remove(args) => cmd::remote::rm::run(args)?,
    },
    Command::Repo(cmd) => match cmd {
      RepoCommand::Add(args) => cmd::repo::add::run(args)?,
      RepoCommand::Remove(args) => cmd::repo::rm::run(args)?,
      RepoCommand::List => cmd::repo::ls::run()?,
      RepoCommand::Status => cmd::repo::status::run()?,
      RepoCommand::Primary(args) => cmd::repo::primary::run(args)?,
    },
    Command::Push(args) => cmd::push::run(args)?,
    Command::Pull(args) => cmd::pull::run(args)?,
    Command::Status => cmd::status::run()?,
  }

  Ok(())
}
