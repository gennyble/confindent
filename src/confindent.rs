use crate::{ConfHash, ConfItem, ConfParent, ConfSection};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::Path;
use std::str::FromStr;
use std::string::ParseError;

/// Structure for Reading/Writing configuration
#[derive(Debug, PartialEq)]
pub struct Confindent {
	sections: ConfHash,
}

impl Confindent {
	/// Create an empty configuration
	pub fn new() -> Self {
		Confindent {
			sections: HashMap::new(),
		}
	}

	/// Creates a new configuration from a file
	///
	/// ## Examples
	/// ```
	/// use confindent::Confindent;
	///
	/// let conf = Confindent::from_file("./examples/example.conf");
	/// ```
	pub fn from_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
		let string = fs::read_to_string(path)?;
		Ok(Confindent::from_str(&string).expect("This should not happen"))
	}

	/// Writes configurtion to a file
	///
	/// ## Examples
	/// ```
	/// use confindent::{Confindent, ConfParent};
	///
	/// let mut conf = Confindent::new();
	/// conf.create("Section", "Value");
	/// conf.child_mut("Section")
	///     .unwrap()
	///     .create("SubSection", "Value")
	///     .create("SubSection", "Value");
	///
	/// conf.to_file("example.conf").unwrap();
	/// ```
	pub fn to_file<P: AsRef<Path>>(self, path: P) -> io::Result<()> {
		let mut file = File::create(path)?;
		let conf: String = self.into();

		file.write_all(&conf.into_bytes())
	}

	fn add_section(&mut self, key: String, cs: ConfSection) {
		if self.sections.is_empty() || cs.get_indent_level() == 0 {
			self.sections.insert(key, cs);
			return;
		}

		let mut hashvec: Vec<(&String, &mut ConfSection)> = self.sections.iter_mut().collect();
		let iter = hashvec.iter_mut().rev();

		for (_, sec) in iter {
			if (**sec).get_indent_level() == cs.get_indent_level() - 1 {
				(**sec).children.insert(key, cs);
				return;
			}
		}

		self.sections.insert(key, cs);
	}
}

impl FromStr for Confindent {
	type Err = ParseError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let mut ret = Self::new();

		if s.is_empty() || s.trim_start().is_empty() {
			return Ok(ret);
		}

		let lines = s.lines();

		for line in lines {
			match ConfSection::parse(line) {
				Some((k, v)) => ret.add_section(k, v),
				None => continue,
			}
		}

		Ok(ret)
	}
}

impl ConfParent for Confindent {
	fn get_child<T: Into<String>>(&self, key: T) -> Option<&ConfSection> {
		self.sections.get(&key.into())
	}

	fn get_child_mut<T: Into<String>>(&mut self, key: T) -> Option<&mut ConfSection> {
		self.sections.get_mut(&key.into())
	}

	fn create_child<T: Into<String>>(&mut self, key: T, value: T) -> &mut Self {
		let sec = ConfSection::new(ConfItem::parse(&value.into()), 0, HashMap::new());
		self.sections.insert(key.into(), sec);

		self
	}
}

impl Into<String> for Confindent {
	fn into(self) -> String {
		let mut ret = String::new();

		for (key, child) in self.sections {
			ret.push_str(&format!("\n{}", child.into_string(key)));
		}

		ret.trim().to_owned()
	}
}
