//! A simple configuration reader.
//!
//! This crate tries to make it easy to add a configuration file to your project.
//! It's not the fastest out there, and it *does* make allocations, but I've tried
//! my best to make it easy to use and the docs easy to read.
//!
//! # The Format
//!
//! Confindent was born out of a weird enjoyment of the way OpenSSH structures
//! its configuration files so it is incredibly similar.
//!
//! It's a kind of tree key-value format. It's pretty similar to JSON in that regard.
//! A key is separated from it's value by a space. Your key can be anything, but the
//! first space that appears in a line, after the indent, is where the value begins.
//!
//! You can indent with spaces or tabs, but don't mix them in one file. Really you
//! just need to be sure that the entire indent, all the way from the not-indented
//! root value, is the same, but be careful if you mix. Confindent will return an
//! error telling you that spaces/tabs were mixed and which one it's supposed to
//! be.
//!
//! Let's say you're writing a music player and for some reason you want to use
//! confindent to define that song. The name of the song is Dots and the artist is
//! Jerobeam Fenderson. It's on the album Oscilloscope Music, it's encoded
//! at a bitrate of 320kbps, and it's 3:10 long (190 seconds). You can describe that
//! perfectly like this:
//!
//! ```ignore
//! Song Dots
//! 	Artist Jerobeam Fenderson
//! 	Album Oscilloscope Music
//! 	Length 190
//! 	Bitrate 320
//! ```
//!
//! # Example
//!
//! ```rust
//! use confindent::Confindent;
//!
//! fn main() {
//! 	let conf = Confindent::from_file("examples/songinfo.conf").unwrap();
//! 	let song = conf.child("Song").unwrap();
//! 	let length: usize = song.child_parse("Length").unwrap();
//!
//! 	println!(
//! 		"Now playing {} by {} [{}:{} {}kbps]",
//! 		song.value().unwrap(),
//! 		song.child_value("Artist").unwrap(),
//! 		length / 60, //minutes
//! 		length % 60, //seconds
//! 		song.child_value("Bitrate").unwrap()
//! 	);
//! }
//! ```

mod error;
mod indent;
mod value;

use std::{fs, path::Path, str::FromStr};

pub use error::{ParseError, ParseErrorKind};
use indent::Indent;
pub use value::Value;

/// A parsed configuration file. This struct holds the values with no indentation.
#[derive(Debug, PartialEq)]
pub struct Confindent {
	children: Vec<Value>,
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
		for child in &self.children {
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
		self.children
			.iter()
			.filter(|value| value.key == key.as_ref())
			.collect()
	}

	/// Get the value of a child with the provided key.
	///
	/// See [Value::child_value] for more.
	pub fn child_value<S: AsRef<str>>(&self, key: S) -> Option<&str> {
		self.child(key).map(|child| child.value()).flatten()
	}

	/// Pase the value of a child into your desired type.
	///
	/// Please, see [Value::child_parse] for more.
	pub fn child_parse<S: AsRef<str>, T: FromStr>(&self, key: S) -> Result<T, <T as FromStr>::Err> {
		self.child(key)
			.map(|child| child.parse())
			.unwrap_or("".parse())
	}

	fn push(&mut self, value: Value) -> Result<(), ParseErrorKind> {
		// Handle the easy stuff first
		if Indent::Empty == value.indent {
			self.children.push(value);
			return Ok(());
		} else if self.children.is_empty() {
			return Err(ParseErrorKind::StartedIndented);
		}

		let mut curr = self.children.last_mut().unwrap();
		match value.indent {
			Indent::Tabs(tabsize) => loop {
				match curr.children.last() {
					None => {
						curr.children.push(value);
						break;
					}
					Some(child) => match child.indent {
						Indent::Empty => unreachable!(),
						Indent::Spaces(_) => return Err(ParseErrorKind::TabsWithSpaces),
						Indent::Tabs(level) => {
							if tabsize == level {
								curr.children.push(value);
								break;
							} else {
								curr = curr.children.last_mut().unwrap();
							}
						}
					},
				}
			},
			Indent::Spaces(size) => loop {
				match curr.children.last() {
					None => {
						curr.children.push(value);
						break;
					}
					Some(child) => match child.indent {
						Indent::Empty => unreachable!(),
						Indent::Tabs(_) => return Err(ParseErrorKind::SpacesWithTabs),
						Indent::Spaces(level) => {
							if size == level {
								curr.children.push(value);
								break;
							} else {
								curr = curr.children.last_mut().unwrap();
							}
						}
					},
				}
			},
			_ => unreachable!(),
		}

		Ok(())
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
			let value_op = Value::from_str(line).map_err(|e| add_ln(e, line_number))?;

			match value_op {
				Some(v) => ret.push(v).map_err(|e| add_ln(e, line_number))?,
				None => continue,
			};
		}

		Ok(ret)
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn parses_single() {
		let single = "Key Value";

		assert_eq!(
			Confindent::from_str(single).unwrap(),
			Confindent {
				children: vec![Value::new(Indent::Empty, "Key", "Value")]
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
					Value::new(Indent::Empty, "Key1", "Value1"),
					Value::new(Indent::Empty, "Key2", "Value2")
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
				children: vec![Value {
					indent: Indent::Empty,
					key: "Key1".into(),
					value: Some("Value1".into()),
					children: vec![Value::new(Indent::Tabs(1), "Key2", "Value2")]
				}]
			}
		);
	}

	#[test]
	fn parses_three_allindented() {
		let doubledent = "Key1 Value1\n\tKey2 Value2\n\t\tKey3 Value3";

		assert_eq!(
			Confindent::from_str(doubledent).unwrap(),
			Confindent {
				children: vec![Value {
					indent: Indent::Empty,
					key: "Key1".into(),
					value: Some("Value1".into()),
					children: vec![Value {
						indent: Indent::Tabs(1),
						key: "Key2".into(),
						value: Some("Value2".into()),
						children: vec![Value::new(Indent::Tabs(2), "Key3", "Value3")]
					}]
				}]
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
					Value {
						indent: Indent::Empty,
						key: "Key1".into(),
						value: Some("Value1".into()),
						children: vec![Value::new(Indent::Tabs(1), "Key2", "Value2")]
					},
					Value::new(Indent::Empty, "Key3", "Value3")
				]
			}
		);
	}
}
