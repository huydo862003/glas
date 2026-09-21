//! Colored terminal output and progress indicators

use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;

pub fn print_ok(message: &str) {
  println!("{} {}", "ok".green(), message);
}

pub fn print_skip(message: &str) {
  println!("{} {}", "skip".yellow(), message);
}

pub fn print_info(message: &str) {
  println!("{message}");
}

/// Create a progress bar for iterating over N items
pub fn create_progress_bar(len: u64) -> ProgressBar {
  let progress = ProgressBar::new(len);
  progress.set_style(
    ProgressStyle::with_template("{bar:30} {pos}/{len} {msg}")
      .unwrap()
      .progress_chars("=> "),
  );
  progress
}
