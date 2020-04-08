use crate::confsection::ConfSection;
use std::str::FromStr;

/// Methods for configuration sections with children
pub trait ConfParent {
	/// Get a reference to a child section
	///
	/// ## Example
	/// ```
	/// use std::str::FromStr;
	/// use confindent::{Confindent, ConfParent};
	///
	/// let conf_str = "Section value";
	/// let conf = Confindent::from_str(conf_str).unwrap();
	/// let section = conf.get_child("Section").unwrap();
	/// ```
	fn get_child<T: Into<String>>(&self, key: T) -> Option<&ConfSection>;

	/// Shorthand for [`get_child()`](#method.get_child)
	fn child<T: Into<String>>(&self, key: T) -> Option<&ConfSection> {
		self.get_child(key)
	}

	/// Get a mutable reference to a child section
	///
	/// ## Example
	/// ```
	/// use std::str::FromStr;
	/// use confindent::{Confindent, ConfParent};
	///
	/// let conf_str = "Section value";
	/// let mut conf = Confindent::from_str(conf_str).unwrap();
	/// let mut section = conf.get_child_mut("Section").unwrap();
	/// ```
	fn get_child_mut<T: Into<String>>(&mut self, key: T) -> Option<&mut ConfSection>;

	/// Shorthand for [`get_child_mut()`](#method.get_child_mut)
	fn child_mut<T: Into<String>>(&mut self, key: T) -> Option<&mut ConfSection> {
		self.get_child_mut(key)
	}

	/// Create a child section
	///
	/// ## Example
	/// ```
	/// use confindent::{Confindent, ConfParent};
	///
	/// let mut conf = Confindent::new();
	/// conf.create_child("Key", "Value");
	/// ```
	fn create_child<T: Into<String>>(&mut self, key: T, value: T) -> &mut Self;

	/// Shorthand for [`create_child()`](#method.create_child)
	fn create<T: Into<String>>(&mut self, key: T, value: T) -> &mut Self {
		self.create_child(key, value)
	}

	/// Get the value of a child
	///
	/// ## Example
	/// ```
	/// use std::str::FromStr;
	/// use confindent::{Confindent, ConfParent};
	///
	/// let conf_str = "Section key";
	/// let conf = Confindent::from_str(conf_str).unwrap();
	///
	/// let value: Option<String> = conf.get_child_value("Section");
	/// assert_eq!(value.unwrap(), "key");
	/// ```
	fn get_child_value<T: Into<String>, Y: FromStr>(&self, key: T) -> Option<Y> {
		match self.get_child(key) {
			None => None,
			Some(child) => child.get(),
		}
	}

	/// Shorthand for [`get_child_value()`](#mathod.get_child_value)
	fn child_value<T: Into<String>, Y: FromStr>(&self, key: T) -> Option<Y> {
		self.get_child_value(key)
	}
}
