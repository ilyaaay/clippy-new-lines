use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CargoMessage {
  pub reason: String,
  pub message: Option<Diagnostic>,
}

#[derive(Debug, Deserialize)]
pub struct Diagnostic {
  pub level: String,
  pub message: String,
  pub code: Option<DiagnosticCode>,
  #[serde(default)]
  pub spans: Vec<Span>,
}

#[derive(Debug, Deserialize)]
pub struct DiagnosticCode {
  pub code: String,
}

#[derive(Debug, Deserialize)]
pub struct Span {
  pub file_name: String,
  pub line_start: Option<usize>,
  pub line_end: Option<usize>,
  pub column_start: Option<usize>,
  #[serde(default)]
  pub is_primary: bool,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Problem {
  pub path: String,
  pub line: usize,
  pub column: usize,
  pub level: String,
  pub message: String,
  pub code: String,
}
