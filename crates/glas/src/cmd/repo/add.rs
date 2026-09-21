//! glas repo add
//! Track a repo, optionally symlinking an external path into the workspace

use std::path::{Path, PathBuf};

use crate::cli::RepoAddArgs;
use crate::git;
use crate::glas::Workspace;
use crate::logger;

pub fn run(args: RepoAddArgs) -> anyhow::Result<()> {
  let mut workspace = Workspace::load()?;
  let root = workspace.root().to_path_buf();

  let (name, source_path) = match args.path {
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
    if repo_path.is_symlink() {
      std::fs::remove_file(&repo_path)?;
    } else if repo_path.exists() {
      anyhow::bail!("{} already exists and is not a symlink", repo_path.display());
    }
    create_symlink(&source_path, &repo_path)?;
    logger::print_info(&format!("symlinked {name} to {}", source_path.display()));
  }

  workspace.add_repo(name.clone(), args.force)?;
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
