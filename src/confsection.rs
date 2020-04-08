use crate::confitem::ConfItem;
use crate::confparent::ConfParent;
use crate::ConfHash;
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub struct ConfSection {
	value: ConfItem,
	indent_level: u8,
	pub children: ConfHash,
}

impl ConfSection {
	pub fn new(value: ConfItem, indent_level: u8, children: ConfHash) -> Self {
		ConfSection {
			value,
			indent_level,
			children,
		}
	}

	/// Set the value of this section
	///
	/// ## Example
	/// ```
	/// use confindent::{Confindent, ConfParent};
	///
	/// let mut conf = Confindent::new();
	/// conf.create("Section", "Placeholder");
	///
	/// let section = conf.child_mut("Section").unwrap();
	/// section.set_value("Value");
	///
	/// assert_eq!(section.get::<String>().unwrap(), "Value");
	/// ```
	pub fn set_value<T: Into<String>>(&mut self, value: T) -> &mut Self {
		self.value = ConfItem::parse(&value.into());

		self
	}

	///Shorthand for [`set_value()`](#method.set_value)
	pub fn set<T: Into<String>>(&mut self, value: T) -> &mut Self {
		self.set_value(value)
	}

	/// Get the scalar value of this section
	///
	/// ## Example
	/// ```
	/// use std::str::FromStr;
	/// use confindent::{Confindent, ConfParent};
	///
	/// let conf_str = "Section value";
	/// let conf = Confindent::from_str(conf_str).unwrap();
	/// let section = conf.child("Section").unwrap();
	///
	/// assert_eq!(section.get_value::<String>().unwrap(), "value");
	/// ```
	pub fn get_value<T: FromStr>(&self) -> Option<T> {
		self.value.get()
	}

	/// Shorthand for [`get_value()`](#method.get_value)
	pub fn get<T: FromStr>(&self) -> Option<T> {
		self.get_value()
	}

	/// Get the value in this section as a vector
	///
	/// ## Example
	/// ```
	/// use std::str::FromStr;
	/// use confindent::{Confindent, ConfParent};
	///
	/// let conf_str = "Section 1,2,3";
	/// let conf = Confindent::from_str(conf_str).unwrap();
	///
	/// let section = conf.child("Section").unwrap();
	/// assert_eq!(section.get_vec(), Some(vec![1, 2, 3]));
	/// ```
	pub fn get_vec<T: FromStr>(&self) -> Option<Vec<T>> {
		match self.get::<String>() {
			None => None,
			Some(x) => x
				.split(',')
				.map(|x| x.trim().parse())
				.collect::<Result<Vec<T>, _>>()
				.ok(),
		}
	}

	pub fn into_string(self, key: String) -> String {
		let mut ret = format!("{} {}", key, self.value);

		for (key, child) in self.children {
			let child_str = format!("\n\t{}", child.into_string(key).replace('\n', "\n\t"));
			ret.push_str(&child_str);
		}

		ret
	}

	pub fn parse(s: &str) -> Option<(String, Self)> {
		if s.is_empty() || s.trim_start().is_empty() {
			return None;
		}

		let mut workable: &str = &s;

		let mut indent_level = 0;
		while workable.starts_with('\t') || workable.starts_with("  ") {
			indent_level += 1;

			let offset = if workable.starts_with('\t') { 1 } else { 2 };

			workable = match workable.get(offset..) {
				Some(slice) => slice,
				None => return None,
			};
		}

		let split: Vec<&str> = workable.split(char::is_whitespace).collect();

		let key = match split.get(0) {
			Some(key) => (*key).to_owned(),
			None => return None,
		};

		let value = match split.get(1) {
			Some(value) => ConfItem::parse(value),
			None => ConfItem::Empty,
		};

		Some((key, Self::new(value, indent_level, HashMap::new())))
	}

	pub fn get_indent_level(&self) -> u8 {
		return self.indent_level;
	}
}

impl ConfParent for ConfSection {
	fn get_child<T: Into<String>>(&self, key: T) -> Option<&ConfSection> {
		self.children.get(&key.into())
	}

	fn get_child_mut<T: Into<String>>(&mut self, key: T) -> Option<&mut ConfSection> {
		self.children.get_mut(&key.into())
	}

	fn create_child<T: Into<String>>(&mut self, key: T, value: T) -> &mut Self {
		let sec = ConfSection::new(
			ConfItem::parse(&value.into()),
			self.indent_level + 1,
			HashMap::new(),
		);
		self.children.insert(key.into(), sec);
		self
	}
}
