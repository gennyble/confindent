mod error;
mod indent;
mod value;

use std::{fs, path::Path, str::FromStr};

pub use error::{ParseError, ParseErrorKind};
use indent::Indent;
pub use value::Value;

#[derive(Debug, PartialEq)]
pub struct Confindent {
	children: Vec<Value>,
}

impl Confindent {
	pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ParseError> {
		let string = fs::read_to_string(path).map_err(|_| ParseError {
			line: 0,
			kind: ParseErrorKind::FileReadError,
		})?;

		Confindent::from_str(&string)
	}

	pub fn child<S: AsRef<str>>(&self, key: S) -> Option<&Value> {
		for child in &self.children {
			if child.key == key.as_ref() {
				return Some(child);
			}
		}

		None
	}

	pub fn children<S: AsRef<str>>(&self, key: S) -> Vec<&Value> {
		self.children
			.iter()
			.filter(|value| value.key == key.as_ref())
			.collect()
	}

	pub fn child_value<S: AsRef<str>>(&self, key: S) -> Option<&str> {
		self.child(key).map(|child| child.value()).flatten()
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
				children: vec![Value::new(Indent::Empty, "Key", Some("Value".into()))]
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
					Value::new(Indent::Empty, "Key1", Some("Value1".into())),
					Value::new(Indent::Empty, "Key2", Some("Value2".into()))
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
					children: vec![Value::new(Indent::Tabs(1), "Key2", Some("Value2".into()))]
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
						children: vec![Value::new(Indent::Tabs(2), "Key3", Some("Value3".into()))]
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
						children: vec![Value::new(Indent::Tabs(1), "Key2", Some("Value2".into()))]
					},
					Value::new(Indent::Empty, "Key3", Some("Value3".into()))
				]
			}
		);
	}
}
