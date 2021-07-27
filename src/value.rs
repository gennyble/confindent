use std::str::FromStr;

use crate::{
	error::{ParseErrorKind, ValueParseError},
	indent::Indent,
};

/// A parsed line of a configuration file.
#[derive(Clone, Debug, PartialEq)]
pub struct Value {
	pub(crate) indent: Indent,
	pub(crate) key: String,
	pub(crate) value: Option<String>,
	pub(crate) children: Vec<Value>,
}

impl Value {
	#[allow(dead_code)] //used heavily in tests
	pub(crate) fn new<K: Into<String>, V: Into<String>>(indent: Indent, key: K, value: V) -> Self {
		let value = value.into();

		Self {
			indent,
			key: key.into(),
			value: if value.is_empty() { None } else { Some(value) },
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
	/// let white_end = Value::whitespace_end_index(string);
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

	/// Parse a line to get a Value from it. This is not the function from FromStr, but just
	/// an associated function with the same name. This is because of the differing return type.
	///
	/// # Returns
	///
	/// - `Ok(None)` if the string is empty or only whitespace.
	/// - `Ok(Some(Value))` if parsing went well. Indent is valid and there's at least a key.
	/// - `Err(ParseErrorKind::MixedIndent)` if indentation was bad.
	pub(crate) fn from_str(line: &str) -> Result<Option<Self>, ParseErrorKind> {
		if line.is_empty() || line.trim().is_empty() {
			return Ok(None);
		}

		let (whitespace, expression) = line.split_at(Self::whitespace_end_index(line));
		let (key, value) = match expression.split_once(' ') {
			None => (expression.to_owned(), None),
			Some((key, value)) if value.is_empty() => (key.to_owned(), None),
			Some((key, value)) => (key.to_owned(), Some(value.to_owned())),
		};

		Ok(Some(Self {
			indent: whitespace.parse()?,
			key,
			value,
			children: vec![],
		}))
	}

	/// Get the first child with the provided key
	///
	/// # Example
	///
	/// ```rust
	/// use confindent::Confindent;
	///
	/// let conf: Confindent = "rootkey rootvalue\n\tchild value".parse().unwrap();
	/// let section = conf.child("rootkey").unwrap();
	/// let grandchild = section.child("child").unwrap();
	///
	/// assert_eq!(section.value(), Some("rootvalue"));
	/// assert_eq!(grandchild.value(), Some("value"));
	/// ```
	pub fn child<S: AsRef<str>>(&self, key: S) -> Option<&Value> {
		for child in &self.children {
			if child.key == key.as_ref() {
				return Some(child);
			}
		}
		None
	}

	/// Get every child that is a direct descendant of this value with the provided name.
	///
	/// # Example
	///
	/// ```rust
	/// use confindent::Confindent;
	///
	/// let confstr = "child value\n\tgrandchild grandvalue\n\tgrandchild morevalue";
	///
	/// let conf: Confindent = confstr.parse().unwrap();
	/// let section = conf.child("child").unwrap();
	/// let children = section.children("grandchild");
	///
	/// assert_eq!(children[0].value(), Some("grandvalue"));
	/// assert_eq!(children[1].value(), Some("morevalue"));
	/// ```
	pub fn children<S: AsRef<str>>(&self, key: S) -> Vec<&Value> {
		self.children
			.iter()
			.filter(|value| value.key == key.as_ref())
			.collect()
	}

	/// Get the value of the first child with the provided key.
	///
	/// This is shorthand for calling [child](Value::child) and then [value](Value::value).
	///
	/// # Returns
	///
	/// The value of the child if both the child is present and it has a value, otherwise None.
	///
	/// # Example
	///
	/// ```rust
	/// use confindent::Confindent;
	///
	/// let conf: Confindent = "child value\n\tgrandchild grandvalue".parse().unwrap();
	/// let section = conf.child("child").unwrap();
	///
	/// assert_eq!(section.child_value("grandchild"), Some("grandvalue"));
	/// ```
	pub fn child_value<S: AsRef<str>>(&self, key: S) -> Option<&str> {
		self.child(key).map(|child| child.value()).flatten()
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

	/// Gets the parsed value of a child that matches the key.
	///
	/// Shorthand for [child](Value::child) and then [parse](Value::parse).
	///
	/// # Example
	///
	/// ```rust
	/// use confindent::Confindent;
	///
	/// let conf: Confindent = "Host host\n\tPort 22".parse().unwrap();
	/// let host = conf.child("Host").unwrap();
	///
	/// assert_eq!(host.child_parse("Port"), Ok(22));
	/// ```
	pub fn child_parse<S: AsRef<str>, T: FromStr>(&self, key: S) -> Result<T, ValueParseError<T>> {
		self.child(key)
			.map(|child| child.parse())
			.unwrap_or(Err(ValueParseError::NoValue))
	}

	/// The same as [value](Value::value) but parses the value into your type. The type you're trying to
	/// parse to must implement [FromStr](std::str::FromStr).
	///
	/// You can think of this as shorthand for
	/// getting the value and trying to parse with `.parse()` because that's exactly what it's doing internally.
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

		assert_eq!(Value::whitespace_end_index(nowhite), 0);
		assert_eq!(Value::whitespace_end_index(tab), 1);
		assert_eq!(Value::whitespace_end_index(space), 1);
		assert_eq!(Value::whitespace_end_index(twospace), 2);
		assert_eq!(Value::whitespace_end_index(mixed), 2);
	}

	#[test]
	fn from_str() {
		let empty = "";
		assert_eq!(Value::from_str(empty).unwrap(), None);

		let noindent = "Key Value";
		let noindent_novalue = "Key";

		assert_eq!(
			Value::from_str(noindent).unwrap().unwrap(),
			Value::new(Indent::Empty, "Key", "Value")
		);
		assert_eq!(
			Value::from_str(noindent_novalue).unwrap().unwrap(),
			Value::new(Indent::Empty, "Key", "")
		);

		let indent = "\tKey Value";
		let indent_novalue = "\tKey";

		assert_eq!(
			Value::from_str(indent).unwrap().unwrap(),
			Value::new(Indent::Tabs(1), "Key", "Value")
		);
		assert_eq!(
			Value::from_str(indent_novalue).unwrap().unwrap(),
			Value::new(Indent::Tabs(1), "Key", "")
		);

		let mixed = " \tKey Value";
		assert_eq!(
			Value::from_str(mixed).unwrap_err(),
			ParseErrorKind::MixedIndent
		);
	}
}
