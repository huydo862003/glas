//! glas init
//! Create a new workspace in the current directory

use crate::cli::InitArgs;
use crate::glas::Workspace;

pub fn run(args: InitArgs) -> anyhow::Result<()> {
  let cwd = std::env::current_dir()?;
  Workspace::init(&cwd, &args.meta_repo)?;

  println!("initialized glas workspace at {}", cwd.display());
  println!("meta-repo name: {}", args.meta_repo);
  Ok(())
}
