use std::{
    ops::{Add, AddAssign},
    str::FromStr,
};

use crate::error::{Error, ErrorKind};

#[derive(Clone, Debug, PartialEq)]
pub enum Indent {
    Empty,
    Tabs(usize),
    Spaces(usize),
}

impl Add<usize> for Indent {
    type Output = Indent;

    fn add(self, rhs: usize) -> Self::Output {
        match self {
            Indent::Tabs(cnt) => Indent::Tabs(cnt + rhs),
            Indent::Spaces(cnt) => Indent::Spaces(cnt + rhs),
            Indent::Empty => Indent::Empty,
        }
    }
}

impl AddAssign<usize> for Indent {
    fn add_assign(&mut self, rhs: usize) {
        match self {
            Indent::Tabs(cnt) => *cnt += rhs,
            Indent::Spaces(cnt) => *cnt += rhs,
            Indent::Empty => (),
        }
    }
}

impl FromStr for Indent {
    type Err = ErrorKind;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Ok(Indent::Empty);
        } else if !s.trim().is_empty() {
            // if we're not only dealing with whitespace, this is a programming error
            panic!("Tried to pass Indent::from_str a string with something other than tabs/spaces");
        }

        let mut chars = s.chars();
        let mut indent = match chars.next() {
            Some(' ') => Indent::Spaces(1),
            Some('\t') => Indent::Tabs(1),
            _ => unreachable!(),
        };

        for ch in chars {
            match ch {
                ' ' => match indent {
                    Indent::Spaces(_) => indent += 1,
                    Indent::Tabs(_) => return Err(ErrorKind::MixedIndent),
                    _ => unreachable!(),
                },
                '\t' => match indent {
                    Indent::Tabs(_) => indent += 1,
                    Indent::Spaces(_) => return Err(ErrorKind::MixedIndent),
                    _ => unreachable!(),
                },
                _ => unreachable!(),
            }
        }

        Ok(indent)
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
        assert_eq!(Indent::from_str(onetab).unwrap(), Indent::Tabs(1));
        assert_eq!(Indent::from_str(onespace).unwrap(), Indent::Spaces(1));

        let twotab = "\t\t";
        let twospace = "  ";

        assert_eq!(Indent::from_str(twotab).unwrap(), Indent::Tabs(2));
        assert_eq!(Indent::from_str(twospace).unwrap(), Indent::Spaces(2));

        let mixedwhitespace = "\t ";

        assert_eq!(
            Indent::from_str(mixedwhitespace).unwrap_err(),
            ErrorKind::MixedIndent
        )
    }

    #[test]
    #[should_panic]
    fn not_whitespace_panic() {
        Indent::from_str(" a").unwrap();
    }
}
