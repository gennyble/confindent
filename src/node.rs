use std::str::FromStr;

use crate::{indent::Indent, line::Line, Entry, EntryIterator, EntryIteratorMut, ValueParseError};

//FIXME: gen- Can we/should we try to drop sized here?
// Sized is required by the two funcitons at the end of the file
pub trait Node: Sized {
	//TODO: gen- Do we really want to do this?
	fn lines(&self) -> &[Line];
	fn lines_mut(&mut self) -> &mut [Line];
	fn indent(&self) -> Indent;

	fn entries(&self) -> EntryIterator;
	fn entries_mut(&mut self) -> EntryIteratorMut;

	fn push_line(&mut self, line: Line);
	fn insert_line(&mut self, idx: usize, line: Line);

	//TODO: gen- Docs
	fn get<S: AsRef<str>>(&self, path: S) -> Option<&str> {
		self.get_delim(path, '/')
	}

	//TODO: gen- Docs
	fn get_parse<S: AsRef<str>, T: FromStr>(&self, path: S) -> Result<T, ValueParseError<T>> {
		self.get_delim_parse(path, '/')
	}

	//TODO: gen- Docs
	fn get_delim<S: AsRef<str>>(&self, path: S, delimeter: char) -> Option<&str> {
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

	//TODO: gen- Docs
	fn get_delim_parse<S: AsRef<str>, T: FromStr>(
		&self,
		path: S,
		delimeter: char,
	) -> Result<T, ValueParseError<T>> {
		self.get_delim(path, delimeter)
			.map(|child| child.parse().map_err(|e| ValueParseError::ParseError(e)))
			.unwrap_or(Err(ValueParseError::NoValue))
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
	fn has_child<S: AsRef<str>>(&self, key: S) -> bool {
		self.entries().any(|value| value.key == key.as_ref())
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
	fn child<S: AsRef<str>>(&self, key: S) -> Option<&Entry> {
		self.entries().find(|value| value.key == key.as_ref())
	}

	//TODO: gen- Better docs?
	/// Like [Node::child()] but mutable
	fn child_mut<S: AsRef<str>>(&mut self, key: S) -> Option<&mut Entry> {
		self.entries_mut().find(|value| value.key == key.as_ref())
	}

	/// Get the value of the first child with the provided key.
	///
	/// This is shorthand for calling [child](Entry::child) and then [value](Entry::value).
	///
	/// # Returns
	///
	/// The value of the child if it's present and it has a value, otherwise None.
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
	fn child_value<S: AsRef<str>>(&self, key: S) -> Option<&str> {
		self.child(key).and_then(|child| child.value())
	}

	/// Gets the parsed value of a child that matches the key.
	///
	/// Shorthand for [child](Entry::child) and then [parse](Entry::parse).
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
	fn child_parse<S: AsRef<str>, T: FromStr>(&self, key: S) -> Result<T, ValueParseError<T>> {
		self.child(key)
			.map(|child| child.parse())
			.unwrap_or(Err(ValueParseError::NoValue))
	}

	/// Get the value of the first child with the provided key.
	///
	/// This is similar to [Entry::child_value] but it clones the string instead of returning
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
	fn child_owned<S: AsRef<str>>(&self, key: S) -> Option<String> {
		self.child_value(key).map(<_>::to_owned)
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
	fn children<S: AsRef<str>>(&self, key: S) -> Vec<&Entry> {
		self.entries()
			.filter(|value| value.key == key.as_ref())
			.collect()
	}

	//TODO: gen- Docs
	fn children_mut<S: AsRef<str>>(&mut self, key: S) -> Vec<&mut Entry> {
		self.entries_mut()
			.filter(|value| value.key == key.as_ref())
			.collect()
	}

	fn push_entry<V: Into<Entry>>(&mut self, value: V) {
		//FIXME: gen- Should not just pick tabs here. Maybe some configurable global?
		let indent = find_child_indent(self).unwrap_or(Indent::TAB);
		let idx = last_non_blank_child(self);

		let mut val = value.into();
		val.indent = indent;
		self.insert_line(idx, Line::Entry(val));
	}

	fn push_comment<C: Into<String>>(&mut self, comment: C) {
		//FIXME: gen- Should not just pick tabs here. Maybe some configurable global?
		let indent = find_child_indent(self).unwrap_or(Indent::TAB);
		let idx = last_non_blank_child(self);

		self.insert_line(
			idx,
			Line::Comment {
				indent,
				comment: comment.into(),
			},
		);
	}
}

fn find_child_indent<N: Node>(node: &N) -> Option<Indent> {
	let first_child = node.lines().iter().find(|l| !l.is_blank());

	match first_child {
		None => {
			if let Indent::Empty = node.indent() {
				None
			} else {
				// We check if we're empty; we shouldn't hit the default here
				Some(node.indent().indent_or_default(Indent::TAB))
			}
		}
		Some(Line::Comment { indent, .. }) => Some(*indent),
		Some(Line::Entry(val)) => Some(val.indent),
		Some(Line::Blank(_)) => unreachable!(),
	}
}

fn last_non_blank_child<N: Node>(node: &N) -> usize {
	node.lines()
		.iter()
		.enumerate()
		.rev()
		.find(|l| !l.1.is_blank())
		.map(|(idx, _)| idx + 1)
		.unwrap_or(0)
}
