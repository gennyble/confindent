use core::fmt;
use std::{
	ops::{Add, AddAssign},
	str::FromStr,
};

use crate::error::ParseErrorKind;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Indent {
	Empty,
	Tabs {
		count: usize,
		/// The number of tabs added from the last indent
		delta: usize,
	},
	Spaces {
		count: usize,
		/// The number of spaces added from the last indent
		delta: usize,
	},
}

impl Indent {
	/// Fill in this indent's delta using `other` as a reference.
	pub(crate) fn delta_from(&mut self, other: &Indent) -> Result<(), ParseErrorKind> {
		match self {
			Indent::Empty => match other {
				Indent::Empty => Ok(()),
				_ => Err(ParseErrorKind::StartedIndented),
			},
			Indent::Tabs { count, delta } => match other {
				Indent::Empty => {
					*delta = *count;
					Ok(())
				}
				Indent::Tabs {
					count: other_count, ..
				} => {
					let diff = *other_count as isize - *count as isize;
					*delta = diff.unsigned_abs();
					Ok(())
				}
				Indent::Spaces { .. } => Err(ParseErrorKind::SpacesWithTabs),
			},
			Indent::Spaces { count, delta } => match other {
				Indent::Empty => {
					*delta = *count;
					Ok(())
				}
				Indent::Tabs { .. } => Err(ParseErrorKind::TabsWithSpaces),
				Indent::Spaces {
					count: other_count, ..
				} => {
					let diff = *other_count as isize - *count as isize;
					*delta = diff.unsigned_abs();
					Ok(())
				}
			},
		}
	}
}

impl Add<usize> for Indent {
	type Output = Indent;

	fn add(self, rhs: usize) -> Self::Output {
		match self {
			Indent::Tabs { count, .. } => Indent::Tabs {
				count: count + rhs,
				delta: rhs,
			},
			Indent::Spaces { count, .. } => Indent::Spaces {
				count: count + rhs,
				delta: rhs,
			},
			Indent::Empty => Indent::Empty,
		}
	}
}

impl AddAssign<usize> for Indent {
	fn add_assign(&mut self, rhs: usize) {
		match self {
			Indent::Tabs { count, delta } => {
				*count += rhs;
				*delta = rhs;
			}
			Indent::Spaces { count, delta } => {
				*count += rhs;
				*delta = rhs;
			}
			_ => (),
		}
	}
}

impl FromStr for Indent {
	type Err = ParseErrorKind;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		if s.is_empty() {
			return Ok(Indent::Empty);
		} else if !s.trim().is_empty() {
			// if we're not only dealing with whitespace, this is a programming error
			panic!("Tried to pass Indent::from_str a string with something other than tabs/spaces");
		}

		let mut chars = s.chars();
		let mut indent = match chars.next() {
			Some(' ') => Indent::Spaces { count: 1, delta: 1 },
			Some('\t') => Indent::Tabs { count: 1, delta: 1 },
			_ => unreachable!(),
		};

		for ch in chars {
			match ch {
				' ' => match indent {
					Indent::Spaces { ref mut count, .. } => *count += 1,
					Indent::Tabs { .. } => return Err(ParseErrorKind::MixedIndent),
					_ => unreachable!(),
				},
				'\t' => match indent {
					Indent::Tabs { ref mut count, .. } => *count += 1,
					Indent::Spaces { .. } => return Err(ParseErrorKind::MixedIndent),
					_ => unreachable!(),
				},
				_ => unreachable!(),
			}
		}

		indent.delta_from(&Indent::Empty)?;

		Ok(indent)
	}
}

impl fmt::Display for Indent {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Indent::Empty => Ok(()),
			Indent::Spaces { count, .. } => {
				let str = " ".repeat(*count);
				write!(f, "{str}")
			}
			Indent::Tabs { count, .. } => {
				let str = "\t".repeat(*count);
				write!(f, "{str}")
			}
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn from_str() {
		let empty = "";
		let onetab = "\t";
		let onespace = " ";

		assert_eq!(Indent::from_str(empty).unwrap(), Indent::Empty);
		assert_eq!(
			Indent::from_str(onetab).unwrap(),
			Indent::Tabs { count: 1, delta: 1 }
		);
		assert_eq!(
			Indent::from_str(onespace).unwrap(),
			Indent::Spaces { count: 1, delta: 1 }
		);

		let twotab = "\t\t";
		let twospace = "  ";

		assert_eq!(
			Indent::from_str(twotab).unwrap(),
			Indent::Tabs { count: 2, delta: 2 }
		);
		assert_eq!(
			Indent::from_str(twospace).unwrap(),
			Indent::Spaces { count: 2, delta: 2 }
		);

		let mixedwhitespace = "\t ";

		assert_eq!(
			Indent::from_str(mixedwhitespace).unwrap_err(),
			ParseErrorKind::MixedIndent
		)
	}

	#[test]
	#[should_panic]
	fn not_whitespace_panic() {
		Indent::from_str(" a").unwrap();
	}

	#[test]
	fn delta_correct() {
		let empty = Indent::Empty;
		let mut tab1 = Indent::Tabs { count: 1, delta: 0 };
		let mut tab3 = Indent::Tabs { count: 3, delta: 0 };

		tab1.delta_from(&empty).unwrap();
		tab3.delta_from(&tab1).unwrap();

		assert_eq!(tab1, Indent::Tabs { count: 1, delta: 1 });
		assert_eq!(tab3, Indent::Tabs { count: 3, delta: 2 });
	}
}
