use std::error::Error as StdError;
use std::fmt::{self, Debug};
use std::str::FromStr;

/// What kind of error happened? Oh, ParseErrorKind of error.
#[derive(Debug, PartialEq)]
pub enum ParseErrorKind {
	StartedIndented,
	MixedIndent,
	TabsWithSpaces,
	SpacesWithTabs,
	FileReadError,
}

/// Our main error type.
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

/// Error returned when parsing a value fails
///
/// ValueParseError will implement `Display`, `Debug`, and `PartialEq` as long
/// long as `<T as FromStr>::Err` implements them.

pub enum ValueParseError<T: FromStr> {
	/// There was no value present to even try and parse
	NoValue,
	/// A value was present but the parse failed. The error is in this enum tuple.
	ParseError(<T as FromStr>::Err),
}

impl<T: FromStr> fmt::Display for ValueParseError<T>
where
	<T as FromStr>::Err: fmt::Display,
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			ValueParseError::NoValue => write!(f, "There was no value to parse present"),
			ValueParseError::ParseError(e) => {
				write!(f, "Failed to parse configuration value: {}", e)
			}
		}
	}
}

impl<T: FromStr> fmt::Debug for ValueParseError<T>
where
	<T as FromStr>::Err: fmt::Debug,
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			ValueParseError::NoValue => f.debug_tuple("NoValue").finish(),
			ValueParseError::ParseError(e) => f.debug_tuple("ParseError").field(e).finish(),
		}
	}
}

impl<T: FromStr> PartialEq for ValueParseError<T>
where
	<T as FromStr>::Err: PartialEq,
{
	fn eq(&self, other: &Self) -> bool {
		if self == &Self::NoValue && other == &Self::NoValue {
			true
		} else if let Self::ParseError(e0) = self {
			if let Self::ParseError(e1) = other {
				e0 == e1
			} else {
				false
			}
		} else {
			false
		}
	}
}
