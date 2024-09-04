use std::{fmt, str::FromStr};

use crate::{
	error::{ParseErrorKind, ValueParseError},
	indent::Indent,
	line::Line,
	node::Node,
	EntryIterator, EntryIteratorMut,
};

/// A parsed line of a configuration file.
#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
	pub(crate) indent: Indent,
	pub(crate) key: String,
	pub(crate) value: Option<String>,
	pub(crate) children: Vec<Line>,
}

impl Entry {
	#[allow(dead_code)] //used heavily in tests
	pub(crate) fn from_parts<K: Into<String>, V: Into<String>>(
		indent: Indent,
		key: K,
		value: V,
	) -> Self {
		let value = value.into();

		Self {
			indent,
			key: key.into(),
			value: if value.is_empty() { None } else { Some(value) },
			children: vec![],
		}
	}

	pub fn new<K: Into<String>, V: fmt::Display>(key: K, value: Option<V>) -> Self {
		Self {
			indent: Indent::Empty,
			key: key.into(),
			value: value.map(|v| v.to_string()),
			children: vec![],
		}
	}

	/// Get the index of the end of the whitespace. The return value is the
	/// index of the first non-whitespace character. You can directly use
	/// this value to get a slice of only the whitespace. Like so:
	///
	/// ```ignore
	/// use confindent::Value;
	///
	/// let string = "\t\tKey Value";
	/// let white_end = Entry::whitespace_end_index(string);
	/// let (white, data) = string.split_at(white_end);
	///
	/// assert_eq!(white, "\t\t");
	/// assert_eq!(data, "Key Value");
	/// ```
	pub(crate) fn whitespace_end_index(s: &str) -> usize {
		let mut iter = s.char_indices();

		loop {
			match iter.next() {
				Some((_, c)) if c.is_ascii_whitespace() => continue,
				Some((i, c)) if !c.is_ascii_whitespace() => return i,
				_ => return 0,
			}
		}
	}

	pub(crate) fn split_whitespace(s: &str) -> Result<(Indent, &str), ParseErrorKind> {
		let (whitespace, other) = s.split_at(Self::whitespace_end_index(s));
		Ok((whitespace.parse()?, other))
	}

	pub(crate) fn from_indent_str(indent: Indent, line: &str) -> Self {
		let (key, value) = match line.split_once(' ') {
			None => (line.to_owned(), None),
			Some((key, value)) if value.is_empty() => (key.to_owned(), None),
			Some((key, value)) => (key.to_owned(), Some(value.to_owned())),
		};

		Self {
			indent,
			key,
			value,
			children: vec![],
		}
	}

	pub(crate) fn from_str(line: &str) -> Result<Self, ParseErrorKind> {
		let (white, expr) = Self::split_whitespace(line)?;
		Ok(Entry::from_indent_str(white, expr))
	}

	/// Gets the contained value.
	///
	/// # Returns
	///
	/// The value if there is one, otherwise None
	///
	/// # Example
	///
	/// ```rust
	/// use confindent::Confindent;
	///
	/// let conf: Confindent = "child value".parse().unwrap();
	/// let section = conf.child("child").unwrap();
	///
	/// assert_eq!(section.value(), Some("value"));
	/// ```
	pub fn value(&self) -> Option<&str> {
		self.value.as_deref()
	}

	//TODO: docs
	pub fn value_mut(&mut self) -> Option<&mut String> {
		self.value.as_mut()
	}

	/// Gets, and clones, the contained value.
	///
	/// # Returns
	///
	/// The value as a `String` if there is one, otherwise None
	///
	/// # Example
	///
	/// ```rust
	/// use confindent::Confindent;
	///
	/// let conf: Confindent = "child value".parse().unwrap();
	/// let section = conf.child("child").unwrap();
	///
	/// assert_eq!(section.value_owned(), Some(String::from("value")));
	/// ```
	pub fn value_owned(&self) -> Option<String> {
		self.value.clone()
	}

	/// The same as [value](Entry::value) but parses the value into your type. The type you're trying to
	/// parse to must implement [FromStr](std::str::FromStr).
	///
	/// You can think of this as shorthand for
	/// getting the value and trying to parse with `.parse()` because that's exactly what is happening internally.
	///
	/// # Example
	///
	/// ```rust
	/// use confindent::Confindent;
	///
	/// let conf: Confindent = "Port 22".parse().unwrap();
	///
	/// assert_eq!(conf.child("Port").unwrap().parse(), Ok(22));
	/// ```
	pub fn parse<T: FromStr>(&self) -> Result<T, ValueParseError<T>> {
		self.value
			.as_ref()
			.map(|child| child.parse().map_err(|e| ValueParseError::ParseError(e)))
			.unwrap_or(Err(ValueParseError::NoValue))
	}

	pub fn parse_opt<T: FromStr>(&self) -> Option<Result<T, ValueParseError<T>>> {
		self.value
			.as_ref()
			.map(|child| child.parse().map_err(|e| ValueParseError::ParseError(e)))
	}

	pub(crate) fn last_value_has_grandchildren(&self) -> bool {
		match self.entries().last() {
			None => false,
			Some(child) => child.entries().last().is_some(),
		}
	}
}

impl Node for Entry {
	fn lines(&self) -> &[Line] {
		&self.children
	}

	fn lines_mut(&mut self) -> &mut [Line] {
		&mut self.children
	}

	fn indent(&self) -> Indent {
		self.indent
	}

	fn entries(&self) -> EntryIterator {
		EntryIterator {
			inner: self.children.iter(),
		}
	}

	fn entries_mut(&mut self) -> EntryIteratorMut {
		EntryIteratorMut {
			inner: self.children.iter_mut(),
		}
	}

	fn push_line(&mut self, line: Line) {
		self.children.push(line);
	}

	fn insert_line(&mut self, idx: usize, line: Line) {
		self.children.insert(idx, line);
	}
}

impl fmt::Display for Entry {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let Entry {
			indent,
			key,
			value,
			children,
		} = self;

		if let Some(value) = value {
			writeln!(f, "{indent}{key} {value}")?;
		} else {
			writeln!(f, "{indent}{key}")?;
		}

		for child in children {
			write!(f, "{child}")?;
		}

		Ok(())
	}
}

impl<K: fmt::Display, V: fmt::Display> From<(K, V)> for Entry {
	fn from(value: (K, V)) -> Self {
		Entry {
			indent: Indent::Empty,
			key: value.0.to_string(),
			value: Some(value.1.to_string()),
			children: vec![],
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn whitespace_end_index() {
		let nowhite = "Key Value";
		let tab = "\tKey Value";
		let space = " Key Value";
		let twospace = "  Key Value";
		let mixed = "\t Key Value";

		assert_eq!(Entry::whitespace_end_index(nowhite), 0);
		assert_eq!(Entry::whitespace_end_index(tab), 1);
		assert_eq!(Entry::whitespace_end_index(space), 1);
		assert_eq!(Entry::whitespace_end_index(twospace), 2);
		assert_eq!(Entry::whitespace_end_index(mixed), 2);
	}

	#[test]
	fn from_str() {
		let noindent = "Key Value";
		let noindent_novalue = "Key";

		assert_eq!(
			Entry::from_str(noindent).unwrap(),
			Entry::from_parts(Indent::Empty, "Key", "Value")
		);
		assert_eq!(
			Entry::from_str(noindent_novalue).unwrap(),
			Entry::from_parts(Indent::Empty, "Key", "")
		);

		let indent = "\tKey Value";
		let indent_novalue = "\tKey";

		assert_eq!(
			Entry::from_str(indent).unwrap(),
			Entry::from_parts(Indent::Tabs { count: 1, delta: 1 }, "Key", "Value")
		);
		assert_eq!(
			Entry::from_str(indent_novalue).unwrap(),
			Entry::from_parts(Indent::Tabs { count: 1, delta: 1 }, "Key", "")
		);

		let mixed = " \tKey Value";
		assert_eq!(
			Entry::from_str(mixed).unwrap_err(),
			ParseErrorKind::MixedIndent
		);
	}

	#[test]
	fn no_indent_only_key() {
		let value = Entry::from_parts(Indent::Empty, "Key", "");
		let expected = "Key\n";

		assert_eq!(value.to_string(), expected)
	}

	#[test]
	fn no_indent_with_value() {
		let value = Entry::from_parts(Indent::Empty, "Key", "Value");
		let expected = "Key Value\n";

		assert_eq!(value.to_string(), expected)
	}

	#[test]
	fn no_indent_with_value_children() {
		let value = Entry {
			indent: Indent::Empty,
			key: "Key".into(),
			value: Some("Value".into()),
			children: vec![Line::Entry(Entry::from_parts(
				Indent::Tabs { count: 1, delta: 1 },
				"ChildKey",
				"Value",
			))],
		};

		let expected = "Key Value\n\tChildKey Value\n";

		assert_eq!(value.to_string(), expected)
	}
}
