use std::error::Error as StdError;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum ErrorKind {
    StartedIndented,
    MixedIndent,
    TabsWithSpaces,
    SpacesWithTabs,
}

#[derive(Debug, PartialEq)]
pub struct Error {
    line: usize,
    kind: ErrorKind,
}

impl StdError for Error {}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            ErrorKind::StartedIndented => {
                write!(
                    f,
                    "Cannot start document with an indented section. Line {}",
                    self.line
                )
            }
            ErrorKind::MixedIndent => {
                write!(
                    f,
                    "Indent mixed between tabs and spaces on line {}",
                    self.line
                )
            }
            ErrorKind::TabsWithSpaces => {
                write!(f, "Tabular indent in space block. Line {}", self.line)
            }
            ErrorKind::SpacesWithTabs => {
                write!(f, "Space indent in tab block. Line {}", self.line)
            }
        }
    }
}
