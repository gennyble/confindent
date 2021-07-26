use std::error::Error as StdError;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum ParseErrorKind {
    StartedIndented,
    MixedIndent,
    TabsWithSpaces,
    SpacesWithTabs,
    FileReadError,
}

#[derive(Debug, PartialEq)]
pub struct ParseError {
    pub line: usize,
    pub kind: ParseErrorKind,
}

impl StdError for ParseError {}
impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            ParseErrorKind::StartedIndented => {
                write!(
                    f,
                    "Cannot start document with an indented section. Line {}",
                    self.line
                )
            }
            ParseErrorKind::MixedIndent => {
                write!(
                    f,
                    "Indent mixed between tabs and spaces on line {}",
                    self.line
                )
            }
            ParseErrorKind::TabsWithSpaces => {
                write!(f, "Tabular indent in space block. Line {}", self.line)
            }
            ParseErrorKind::SpacesWithTabs => {
                write!(f, "Space indent in tab block. Line {}", self.line)
            }
            ParseErrorKind::FileReadError => {
                write!(f, "Failed to open file!")
            }
        }
    }
}
