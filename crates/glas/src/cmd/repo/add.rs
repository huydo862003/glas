//! glas repo add
//! Track a repo, optionally symlinking an external path into the workspace

use std::path::{Path, PathBuf};

use bitflags::bitflags;

use crate::git;
use crate::glas::Workspace;
use crate::logger;

bitflags! {
  pub struct Flags: u32 {
    const FORCE = 1 << 0;
  }
}

pub fn run(path: Option<String>, flags: Flags) -> anyhow::Result<()> {
  let mut workspace = Workspace::load()?;
  let root = workspace.root().to_path_buf();

  let (name, source_path) = match path {
    Some(path) => {
      let source = PathBuf::from(&path).canonicalize()?;
      let name = source
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .map(|str_name| str_name.to_string())
        .ok_or_else(|| anyhow::anyhow!("could not determine repo name from path"))?;
      (name, source)
    }
    None => {
      let name = workspace.get_current_repo_name()?;
      let source = root.join(&name);
      (name, source)
    }
  };

  let repo_path = root.join(&name);

  if source_path.exists() && !git::is_dir_git_repo(&source_path) {
    anyhow::bail!("{} is not a git repository", source_path.display());
  }

  // Symlink if the repo lives outside the workspace
  if source_path != repo_path {
    if repo_path.exists() || repo_path.is_symlink() {
      std::fs::remove_file(&repo_path)?;
    }
    create_symlink(&source_path, &repo_path)?;
    logger::print_info(&format!("symlinked {name} to {}", source_path.display()));
  }

  workspace.add_repo(name.clone(), flags.contains(Flags::FORCE))?;
  workspace.save()?;

  logger::print_ok(&format!("tracking '{name}'"));
  Ok(())
}

#[cfg(unix)]
fn create_symlink(source: &Path, target: &Path) -> anyhow::Result<()> {
  std::os::unix::fs::symlink(source, target)?;
  Ok(())
}

#[cfg(windows)]
fn create_symlink(source: &Path, target: &Path) -> anyhow::Result<()> {
  std::os::windows::fs::symlink_dir(source, target)?;
  Ok(())
}
