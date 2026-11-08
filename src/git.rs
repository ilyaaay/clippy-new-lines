use std::process::Command;

use anyhow::{Context, bail};

pub fn git_output(args: &[&str]) -> anyhow::Result<String> {
  let output = Command::new("git")
    .args(args)
    .output()
    .with_context(|| format!("failed to run `git {}`", args.join(" ")))?;

  if !output.status.success() {
    bail!(
      "`git {}` failed: {}",
      args.join(" "),
      String::from_utf8_lossy(&output.stderr).trim()
    );
  }

  String::from_utf8(output.stdout).context("git output is not valid UTF-8")
}
