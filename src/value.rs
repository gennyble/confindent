use std::{fmt, str::FromStr};

use crate::{
	error::{ParseErrorKind, ValueParseError},
	indent::Indent,
	line::Line,
	ValueIterator, ValueIteratorMut,
};

/// A parsed line of a configuration file.
#[derive(Clone, Debug, PartialEq)]
pub struct Value {
	pub(crate) indent: Indent,
	pub(crate) key: String,
	pub(crate) value: Option<String>,
	pub(crate) children: Vec<Line>,
}

impl Value {
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

	pub fn get<S: AsRef<str>>(&self, path: S) -> Option<&str> {
		self.get_delim(path, '/')
	}

	pub fn get_parse<S: AsRef<str>, T: FromStr>(&self, path: S) -> Result<T, ValueParseError<T>> {
		self.get_delim_parse(path, '/')
	}

	pub fn get_delim_parse<S: AsRef<str>, T: FromStr>(
		&self,
		path: S,
		delimeter: char,
	) -> Result<T, ValueParseError<T>> {
		self.get_delim(path, delimeter)
			.map(|child| child.parse().map_err(|e| ValueParseError::ParseError(e)))
			.unwrap_or(Err(ValueParseError::NoValue))
	}

	pub fn get_delim<S: AsRef<str>>(&self, path: S, delimeter: char) -> Option<&str> {
		let path = path.as_ref();
		let mut splits = path.split(delimeter);

		let mut current = match splits.next().and_then(|key| self.child(key)) {
			None => return None,
			Some(child) => child,
		};

		for key in splits {
			match current.child(key) {
				None => return None,
				Some(child) => current = child,
			}
		}

		current.value()
	}

	//TODO: docs
	pub fn values(&self) -> ValueIterator {
		ValueIterator {
			inner: self.children.iter(),
		}
	}

	//TODO: docs
	pub fn values_mut(&mut self) -> ValueIteratorMut {
		ValueIteratorMut {
			inner: self.children.iter_mut(),
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
		Ok(Value::from_indent_str(white, expr))
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
		self.values().find(|value| value.key == key.as_ref())
	}

	//TODO: docs
	pub fn child_mut<S: AsRef<str>>(&mut self, key: S) -> Option<&mut Value> {
		self.values_mut().find(|value| value.key == key.as_ref())
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
		self.values()
			.filter(|value| value.key == key.as_ref())
			.collect()
	}

	//TODO: docs
	pub fn children_mut<S: AsRef<str>>(&mut self, key: S) -> Vec<&mut Value> {
		self.values_mut()
			.filter(|value| value.key == key.as_ref())
			.collect()
	}

	/// Check if there are any direct children with the provided key.
	///
	/// # Returns
	///
	/// `true` if there was at least one child with the key, otherwise `false`
	///
	/// # Example
	///
	/// ```rust
	/// use confindent::Confindent;
	///
	/// let confstr = "Host localhost\n\tUseCompression";
	///
	/// let conf: Confindent = confstr.parse().unwrap();
	/// let host = conf.child("Host").unwrap();
	///
	/// assert!(host.has_child("UseCompression"));
	/// ```
	pub fn has_child<S: AsRef<str>>(&self, key: S) -> bool {
		self.values().any(|value| value.key == key.as_ref())
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
		self.child(key).and_then(|child| child.value())
	}

	/// Get the value of the first child with the provided key.
	///
	/// This is similar to [Value::child_value] but it clones the string instead of returning
	/// a reference.
	///
	/// # Returns
	///
	/// The value of the child, cloned to a `String` if both the child and value are present, or None.
	///
	/// # Example
	///
	/// ```rust
	/// use confindent::Confindent;
	///
	/// let conf: Confindent = "key value\n\tchildkey childvalue".parse().unwrap();
	/// let section = conf.child("key").unwrap();
	///
	/// assert_eq!(section.child_owned("childkey"), Some(String::from("childvalue")))
	/// ```
	pub fn child_owned<S: AsRef<str>>(&self, key: S) -> Option<String> {
		self.child_value(key).map(<_>::to_owned)
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
}

impl fmt::Display for Value {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let Value {
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
		let noindent = "Key Value";
		let noindent_novalue = "Key";

		assert_eq!(
			Value::from_str(noindent).unwrap(),
			Value::from_parts(Indent::Empty, "Key", "Value")
		);
		assert_eq!(
			Value::from_str(noindent_novalue).unwrap(),
			Value::from_parts(Indent::Empty, "Key", "")
		);

		let indent = "\tKey Value";
		let indent_novalue = "\tKey";

		assert_eq!(
			Value::from_str(indent).unwrap(),
			Value::from_parts(Indent::Tabs { count: 1, delta: 1 }, "Key", "Value")
		);
		assert_eq!(
			Value::from_str(indent_novalue).unwrap(),
			Value::from_parts(Indent::Tabs { count: 1, delta: 1 }, "Key", "")
		);

		let mixed = " \tKey Value";
		assert_eq!(
			Value::from_str(mixed).unwrap_err(),
			ParseErrorKind::MixedIndent
		);
	}

	#[test]
	fn no_indent_only_key() {
		let value = Value::from_parts(Indent::Empty, "Key", "");
		let expected = "Key\n";

		assert_eq!(value.to_string(), expected)
	}

	#[test]
	fn no_indent_with_value() {
		let value = Value::from_parts(Indent::Empty, "Key", "Value");
		let expected = "Key Value\n";

		assert_eq!(value.to_string(), expected)
	}

	#[test]
	fn no_indent_with_value_children() {
		let value = Value {
			indent: Indent::Empty,
			key: "Key".into(),
			value: Some("Value".into()),
			children: vec![Line::Value(Value::from_parts(
				Indent::Tabs { count: 1, delta: 1 },
				"ChildKey",
				"Value",
			))],
		};

		let expected = "Key Value\n\tChildKey Value\n";

		assert_eq!(value.to_string(), expected)
	}
}
