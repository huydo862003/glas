//! glas init
//! Create a new workspace in the current directory

use std::io::{self, Write};

use crate::glas::Workspace;

pub fn run() -> anyhow::Result<()> {
  let cwd = std::env::current_dir()?;
  let meta_repo = prompt_meta_repo_name()?;
  Workspace::init(&cwd, &meta_repo)?;

  println!("initialized glas workspace at {}", cwd.display());
  println!("meta-repo name: {meta_repo}");
  Ok(())
}

fn prompt_meta_repo_name() -> anyhow::Result<String> {
  print!("meta-repo name [glas]: ");
  io::stdout().flush()?;
  let mut input = String::new();
  io::stdin().read_line(&mut input)?;
  let name = input.trim().to_string();
  Ok(if name.is_empty() {
    "glas".to_string()
  } else {
    name
  })
}
