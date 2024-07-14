use super::{
  parser::{FunParser, ParseBook},
  Book, Name,
};
use crate::{
  diagnostics::{Diagnostics, DiagnosticsConfig},
  imports::PackageLoader,
};
use std::path::Path;

// TODO: Refactor so that we don't mix the two syntaxes here.

/// Reads a file and parses to a definition book.
pub fn load_file_to_book(
  path: &Path,
  package_loader: impl PackageLoader,
  diag: DiagnosticsConfig,
) -> Result<Book, Diagnostics> {
  match path.try_exists() {
    Ok(exists) => {
      if !exists {
        return Err(format!("The file '{}' was not found.", path.display()).into());
      }
      let code = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
      load_to_book(path, &code, package_loader, diag)
    }
    Err(e) => Err(e.to_string().into()),
  }
}

pub fn load_to_book(
  origin: &Path,
  code: &str,
  package_loader: impl PackageLoader,
  diag: DiagnosticsConfig,
) -> Result<Book, Diagnostics> {
  let builtins = ParseBook::builtins();
  let book = do_parse_book(code, origin, builtins)?;
  book.load_imports(package_loader, diag)
}

pub fn do_parse_book(code: &str, origin: &Path, mut book: ParseBook) -> Result<ParseBook, String> {
  book.source = Name::new(origin.to_string_lossy());
  FunParser::new(code, false).parse_book(book).map_err(|e| format!("In {} :\n{}", origin.display(), e))
}

pub fn do_parse_book_default(code: &str, origin: &Path) -> Result<Book, String> {
  do_parse_book(code, origin, ParseBook::builtins())?.to_fun()
}
