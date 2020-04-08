use std::fmt;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub enum ConfItem {
	Empty,
	Text(String),
}

impl ConfItem {
	pub fn parse(s: &str) -> Self {
		ConfItem::Text(s.to_owned())
	}

	pub fn get<T: FromStr>(&self) -> Option<T> {
		match *self {
			ConfItem::Empty => None,
			ConfItem::Text(ref s) => s.parse().ok(),
		}
	}
}

impl fmt::Display for ConfItem {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			ConfItem::Empty => write!(f, ""),
			ConfItem::Text(s) => write!(f, "{}", s),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn confitem_test() {
		let data = "256";
		let item = ConfItem::parse(data);

		assert_eq!(Some(256), item.get::<u16>());
	}
}
