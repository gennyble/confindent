use core::fmt;

use crate::{indent::Indent, Value};

#[derive(Debug, Clone, PartialEq)]
pub enum Line {
	Value(Value),
	Comment { indent: Indent, comment: String },
	Blank(String),
}

impl From<Value> for Line {
	fn from(value: Value) -> Self {
		Line::Value(value)
	}
}

impl From<(Indent, String)> for Line {
	fn from(comment: (Indent, String)) -> Self {
		Line::Comment {
			indent: comment.0,
			comment: comment.1,
		}
	}
}

impl From<(Indent, &str)> for Line {
	fn from(comment: (Indent, &str)) -> Self {
		Line::Comment {
			indent: comment.0,
			comment: comment.1.into(),
		}
	}
}

impl fmt::Display for Line {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Line::Blank(blnk) => write!(f, "{blnk}\n"),
			Line::Comment { indent, comment } => write!(f, "{indent}#{comment}\n"),
			Line::Value(v) => v.fmt(f),
		}
	}
}
