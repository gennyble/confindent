//! A simple configuration reader.
//!
//! This crate tries to make it easy to add a configuration file to your project.
//! It's not the fastest out there, and it *does* make allocations, but I've tried
//! my best to make it easy to use and the docs easy to read.
//!
//! # The Format
//!
//! It's a kind of tree, key-value thing. Lines are key-value pairs, the value
//! starting at the first space after the indent. You can add a child to a value
//! by indenting it with spaces or tabs. Indent the same amount to add another
//! child to that same value. Indent more than you did initially to add a
//! grandchild. Don't mix spaces and tabs. Like this!
//!
//! ```text
//! Root this is the root
//! 	Child I'm a child!
//! 	Child You can have multiple children with the same keys!
//! 		Grandchild I'm a grandchild!
//! ```
//!
//! # Example
//!
//! ```rust
//! use confindent::Confindent;
//! use std::error::Error;
//!
//! fn main() {
//! 	let conf: Confindent = "User gennyble\n\tEmail gen@nyble.dev\n\tID 256".parse().unwrap();
//!
//! 	let user = conf.child("User").unwrap();
//! 	let username = user.value().unwrap();
//! 	let email = user.child_value("Email").unwrap();
//! 	let id: usize = user.child_parse("ID").unwrap();
//!
//! 	println!("User {username}: {id} Contact: {email}");
//! }
//! ```

mod error;
mod indent;
mod line;
mod value;

use std::{fs, path::Path, str::FromStr};

pub use error::{ParseError, ParseErrorKind, ValueParseError};
use indent::Indent;
use line::Line;
pub use value::Value;

/// A parsed configuration file. This struct holds the values with no indentation.
#[derive(Debug, PartialEq)]
pub struct Confindent {
	children: Vec<Line>,
}

impl Confindent {
	/// Tries to read and parse the file at the provided path.
	///
	/// # Returns
	///
	/// A new [Confindent] if the file was read and parsed successfully, or a
	/// [ParseError] if not.
	pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ParseError> {
		let string = fs::read_to_string(path).map_err(|_| ParseError {
			line: 0,
			kind: ParseErrorKind::FileReadError,
		})?;

		Confindent::from_str(&string)
	}

	/// Get a child with the provided key.
	///
	/// See [Value::child] for more.
	pub fn child<S: AsRef<str>>(&self, key: S) -> Option<&Value> {
		for child in self.values() {
			if child.key == key.as_ref() {
				return Some(child);
			}
		}

		None
	}

	/// Get all of the direct children with the provided key.
	///
	/// See [Value::children] for more.
	pub fn children<S: AsRef<str>>(&self, key: S) -> Vec<&Value> {
		self.values()
			.filter(|value| value.key == key.as_ref())
			.collect()
	}

	/// Check if there are any direct children with the provided key.
	///
	/// See [Value::has_child] for more.
	pub fn has_child<S: AsRef<str>>(&self, key: S) -> bool {
		self.values()
			.find(|value| value.key == key.as_ref())
			.is_some()
	}

	/// Get the value of a child with the provided key.
	///
	/// See [Value::child_value] for more.
	pub fn child_value<S: AsRef<str>>(&self, key: S) -> Option<&str> {
		self.child(key).map(|child| child.value()).flatten()
	}

	/// Get the value of a child and clone it.
	///
	/// See [Value::child_owned] for more.
	pub fn child_owned<S: AsRef<str>>(&self, key: S) -> Option<String> {
		self.child_value(key).map(<_>::to_owned)
	}

	/// Pase the value of a child into your desired type.
	///
	/// Please, see [Value::child_parse] for more.
	pub fn child_parse<S: AsRef<str>, T: FromStr>(&self, key: S) -> Result<T, ValueParseError<T>> {
		self.child(key)
			.map(|child| child.parse())
			.unwrap_or(Err(ValueParseError::NoValue))
	}

	pub fn values(&self) -> ValueIterator {
		ValueIterator {
			inner: self.children.iter(),
		}
	}

	pub fn values_mut(&mut self) -> ValueIteratorMut {
		ValueIteratorMut {
			inner: self.children.iter_mut(),
		}
	}

	fn push(&mut self, mut line: Line) -> Result<(), ParseErrorKind> {
		let indent = match &mut line {
			Line::Blank(_) => {
				self.push_last(line);
				return Ok(());
			}
			Line::Value(v) => &mut v.indent,
			Line::Comment { ref mut indent, .. } => indent,
		};

		// Handle the easy stuff first
		if Indent::Empty == *indent {
			self.children.push(line);
			return Ok(());
		} else if self.children.is_empty() {
			return Err(ParseErrorKind::StartedIndented);
		}

		let mut curr = self.values_mut().last().unwrap();
		match indent {
			Indent::Tabs { count: tabsize, .. } => loop {
				match curr.values_mut().last() {
					None => {
						indent.delta_from(&curr.indent)?;
						curr.children.push(line);
						break;
					}
					Some(child) => match child.indent {
						Indent::Empty => unreachable!(),
						Indent::Spaces { .. } => return Err(ParseErrorKind::TabsWithSpaces),
						Indent::Tabs {
							count: child_tabsize,
							..
						} => {
							if *tabsize == child_tabsize {
								indent.delta_from(&child.indent)?;
								curr.children.push(line);
								break;
							} else {
								curr = curr.values_mut().last().unwrap();
							}
						}
					},
				}
			},
			Indent::Spaces { count: spaces, .. } => loop {
				match curr.values().last() {
					None => {
						curr.children.push(line);
						break;
					}
					Some(child) => match child.indent {
						Indent::Empty => unreachable!(),
						Indent::Tabs { .. } => return Err(ParseErrorKind::SpacesWithTabs),
						Indent::Spaces {
							count: child_spaces,
							..
						} => {
							if *spaces == child_spaces {
								indent.delta_from(&child.indent)?;
								curr.children.push(line);
								break;
							} else {
								curr = curr.values_mut().last().unwrap();
							}
						}
					},
				}
			},
			_ => unreachable!(),
		}

		Ok(())
	}

	/// Push the provided [Line] to the last, deepest node
	fn push_last(&mut self, line: Line) {
		let mut curr = match self.values_mut().last() {
			None => {
				self.children.push(line);
				return;
			}
			Some(ln) => ln,
		};

		loop {
			match curr.values_mut().last() {
				None => {
					self.children.push(line);
					return;
				}
				Some(last) => curr = last,
			}
		}
	}
}

impl FromStr for Confindent {
	type Err = ParseError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let mut ret = Self { children: vec![] };
		let lines = s.lines().enumerate();
		let add_ln =
			|e: ParseErrorKind, ln: usize| -> ParseError { ParseError { line: ln, kind: e } };

		for (line_number, line) in lines {
			if blank_line(line) {
				ret.push_last(Line::Blank(line.to_owned()));
				continue;
			}

			let (indent, other) =
				Value::split_whitespace(line).map_err(|e| add_ln(e, line_number))?;

			let line = if let Some(comment) = other.strip_prefix('#') {
				Line::Comment {
					indent,
					comment: comment.into(),
				}
			} else {
				Line::Value(Value::from_str(line).map_err(|e| add_ln(e, line_number))?)
			};

			ret.push(line).map_err(|e| add_ln(e, line_number))?;
		}

		Ok(ret)
	}
}

fn blank_line(s: &str) -> bool {
	for ch in s.chars() {
		if !ch.is_whitespace() {
			return false;
		}
	}
	true
}

pub struct ValueIterator<'a> {
	inner: std::slice::Iter<'a, Line>,
}

impl<'a> Iterator for ValueIterator<'a> {
	type Item = &'a Value;

	fn next(&mut self) -> Option<Self::Item> {
		loop {
			match self.inner.next() {
				None => break None,
				Some(Line::Value(v)) => break Some(v),
				_ => continue,
			}
		}
	}
}

pub struct ValueIteratorMut<'a> {
	inner: std::slice::IterMut<'a, Line>,
}

impl<'a> Iterator for ValueIteratorMut<'a> {
	type Item = &'a mut Value;

	fn next(&mut self) -> Option<Self::Item> {
		loop {
			match self.inner.next() {
				None => break None,
				Some(Line::Value(v)) => break Some(v),
				_ => continue,
			}
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;

	macro_rules! value {
		($indent:expr, $key:expr, $value:expr) => {
			Line::Value(Value::new($indent, $key, $value))
		};
	}

	#[test]
	fn parses_single() {
		let single = "Key Value";

		assert_eq!(
			Confindent::from_str(single).unwrap(),
			Confindent {
				children: vec![value!(Indent::Empty, "Key", "Value")]
			}
		);
	}

	#[test]
	fn parses_double_noindent() {
		let double = "Key1 Value1\nKey2 Value2";

		assert_eq!(
			Confindent::from_str(double).unwrap(),
			Confindent {
				children: vec![
					value!(Indent::Empty, "Key1", "Value1"),
					value!(Indent::Empty, "Key2", "Value2")
				]
			}
		);
	}

	#[test]
	fn parses_double_indented() {
		let doubledent = "Key1 Value1\n\tKey2 Value2";

		assert_eq!(
			Confindent::from_str(doubledent).unwrap(),
			Confindent {
				children: vec![Line::Value(Value {
					indent: Indent::Empty,
					key: "Key1".into(),
					value: Some("Value1".into()),
					children: vec![value!(
						Indent::Tabs { count: 1, delta: 1 },
						"Key2",
						"Value2"
					)]
				})]
			}
		);
	}

	#[test]
	fn parses_three_allindented() {
		let doubledent = "Key1 Value1\n\tKey2 Value2\n\t\tKey3 Value3";

		assert_eq!(
			Confindent::from_str(doubledent).unwrap(),
			Confindent {
				children: vec![Line::Value(Value {
					indent: Indent::Empty,
					key: "Key1".into(),
					value: Some("Value1".into()),
					children: vec![Line::Value(Value {
						indent: Indent::Tabs { count: 1, delta: 1 },
						key: "Key2".into(),
						value: Some("Value2".into()),
						children: vec![value!(
							Indent::Tabs { count: 2, delta: 1 },
							"Key3",
							"Value3"
						)]
					})]
				})]
			}
		);
	}

	#[test]
	fn parses_three() {
		let doubledent = "Key1 Value1\n\tKey2 Value2\nKey3 Value3";

		assert_eq!(
			Confindent::from_str(doubledent).unwrap(),
			Confindent {
				children: vec![
					Line::Value(Value {
						indent: Indent::Empty,
						key: "Key1".into(),
						value: Some("Value1".into()),
						children: vec![value!(
							Indent::Tabs { count: 1, delta: 1 },
							"Key2",
							"Value2"
						)]
					}),
					value!(Indent::Empty, "Key3", "Value3")
				]
			}
		);
	}
}

// Code from the bottom of this page:
// https://doc.rust-lang.org/rustdoc/documentation-tests.html
#[doc = include_str!("../README.md")]
#[cfg(doctest)]
struct ReadmeDocTests;
