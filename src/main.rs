mod diagnostic;
mod git;

use crate::{
  diagnostic::{CargoMessage, Problem},
  git::git_output,
};
use std::{
  collections::{BTreeSet, HashMap},
  io::{self, BufRead},
  path::Path,
};

fn main() -> anyhow::Result<()> {
  let current_git_dir = git_output(&["rev-parse", "--show-toplevel"])?;
  let current_git_dir = current_git_dir.trim();

  let stdin = io::stdin();

  let ranges = get_changed_line_ranges()?;

  let problems = {
    let ranges: &HashMap<String, Vec<(usize, usize)>> = &ranges;

    let mut problems = BTreeSet::new();

    for line in stdin.lock().lines() {
      let line = line?;

      let Ok(message) = serde_json::from_str::<CargoMessage>(&line) else {
        continue;
      };

      if message.reason != "compiler-message" {
        continue;
      }

      let Some(diagnostic) = message.message else {
        continue;
      };

      if diagnostic.level != "warning" && diagnostic.level != "error" {
        continue;
      }

      let code = diagnostic
        .code
        .as_ref()
        .map(|code| code.code.clone())
        .unwrap_or_default();

      for span in &diagnostic.spans {
        if !span.is_primary {
          continue;
        }

        let Some(line_start) = span.line_start else {
          continue;
        };
        let line_end = span.line_end.unwrap_or(line_start);

        let path = get_file_path(&current_git_dir, &span.file_name);
        let Some(file_ranges) = ranges.get(&path) else {
          continue;
        };

        if !file_ranges
          .iter()
          .any(|&(start, end)| line_start <= end && line_end >= start)
        {
          continue;
        }

        problems.insert(Problem {
          path,
          line: line_start,
          column: span.column_start.unwrap_or(0),
          level: diagnostic.level.clone(),
          message: diagnostic.message.clone(),
          code: code.clone(),
        });
        break;
      }
    }

    problems
  };

  if !problems.is_empty() {
    {
      println!("\n\x1b[1;31m❌ Найдены проблемы clippy в новом коде:\x1b[0m\n");

      for problem in problems {
        let suffix = if !problem.code.is_empty() {
          format!(" [{}]", problem.code)
        } else {
          String::new()
        };

        println!(
          "  \x1b[1;31m{}\x1b[0m {}:{}:{}: {}{}",
          problem.level,
          problem.path,
          problem.line,
          problem.column,
          problem.message,
          suffix
        );
      }

      println!(
        "\n\x1b[1;33mИсправьте эти предупреждения. Старые warnings не блокируют коммит.\x1b[0m"
      );
    };
  }

  Ok(())
}

fn get_changed_line_ranges()
-> anyhow::Result<HashMap<String, Vec<(usize, usize)>>> {
  let files = git_output(&[
    "diff",
    "--cached",
    "--name-only",
    "-z",
    "--diff-filter=ACMR",
    "--",
    "*.rs",
  ])?;

  let mut ranges = HashMap::new();

  for path in files.split('\0').filter(|path| !path.is_empty()) {
    let diff = git_output(&["diff", "--cached", "--unified=0", "--", path])?;

    let mut file_ranges = Vec::new();

    for line in diff.lines() {
      if let Some((start, count)) = parse_hunk_header(line)
        && count > 0
      {
        file_ranges.push((start, start + count - 1));
      }
    }

    if !file_ranges.is_empty() {
      ranges.insert(path.to_string(), file_ranges);
    }
  }

  Ok(ranges)
}

fn parse_hunk_header(line: &str) -> Option<(usize, usize)> {
  let rest = line.strip_prefix("@@ -")?;
  let plus = rest.find(" +")?;
  let after = &rest[plus + 2..];
  let after = after.split(" @@").next()?;

  let (start, count) = match after.split_once(',') {
    Some((start, count)) => (start.parse().ok()?, count.parse().ok()?),
    None => (after.parse().ok()?, 1),
  };

  Some((start, count))
}

fn get_file_path(root: &str, file_name: &str) -> String {
  let path = Path::new(file_name);

  if path.is_absolute()
    && let Ok(relative) = path.strip_prefix(root)
  {
    return (&relative.to_string_lossy())
      .replace('\\', "/")
      .trim_start_matches("./")
      .to_string();
  }

  file_name
    .replace('\\', "/")
    .trim_start_matches("./")
    .to_string()
}
