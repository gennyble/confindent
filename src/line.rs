use core::fmt;

use crate::{indent::Indent, Entry};

#[derive(Debug, Clone, PartialEq)]
pub enum Line {
	Entry(Entry),
	Comment { indent: Indent, comment: String },
	Blank(String),
}

impl Line {
	pub fn is_blank(&self) -> bool {
		if let Self::Blank(_) = self {
			true
		} else {
			false
		}
	}
}

impl From<Entry> for Line {
	fn from(entry: Entry) -> Self {
		Line::Entry(entry)
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
			Line::Blank(blnk) => writeln!(f, "{blnk}"),
			Line::Comment { indent, comment } => writeln!(f, "{indent}#{comment}"),
			Line::Entry(v) => v.fmt(f),
		}
	}
}
