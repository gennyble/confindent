use std::str::FromStr;

use crate::{error::ParseErrorKind, indent::Indent};

#[derive(Clone, Debug, PartialEq)]
pub struct Value {
    pub(crate) indent: Indent,
    pub(crate) key: String,
    pub(crate) value: Option<String>,
    pub(crate) children: Vec<Value>,
}

impl Value {
    pub(crate) fn new<K: Into<String>>(indent: Indent, key: K, value: Option<String>) -> Self {
        Self {
            indent,
            key: key.into(),
            value: value,
            children: vec![],
        }
    }

    /// Get the index of the end of the whitespace. The return value is the
    /// index of the first non-whitespace character. You can directly use
    /// this value to get a silice of only the whitespace. Like so:
    ///
    /// ```ignore
    /// use confindent::Value;
    ///
    /// let string = "\t\tKey Value";
    /// let white_end = Value::whitespace_end_index(string);
    /// let (white, data) = string.split_at(white_end);
    ///
    /// assert_eq!(white, "\t\t");
    /// assert_eq!(data, "Key Value");
    /// ```
    pub(crate) fn whitespace_end_index(s: &str) -> usize {
        let mut iter = s.char_indices();

        loop {
            match iter.next() {
                Some((_, c)) if c.is_ascii_whitespace() => continue,
                Some((i, c)) if !c.is_ascii_whitespace() => return i,
                _ => return 0,
            }
        }
    }

    /// Parse a line to get a Value from it.
    ///
    /// # Returns
    ///
    /// - `Ok(None)` if the string is empty or only whitespace.
    /// - `Ok(Some(Value))` if parsing went well. Indent is valid and there's at least a key.
    /// - `Err(Error::MixedIndent(0))` if indentation was bad. That zero there is the line number,
    ///    which this function doesn't know, and is expected to be filled by the Confindent parser.
    pub(crate) fn from_str(line: &str) -> Result<Option<Self>, ParseErrorKind> {
        if line.is_empty() || line.trim().is_empty() {
            return Ok(None);
        }

        let (whitespace, expression) = line.split_at(Self::whitespace_end_index(line));
        let (key, value) = match expression.split_once(' ') {
            None => (expression.to_owned(), None),
            Some((key, value)) => (key.to_owned(), Some(value.to_owned())),
        };

        Ok(Some(Self {
            indent: whitespace.parse()?,
            key,
            value,
            children: vec![],
        }))
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

    pub fn value(&self) -> Option<&str> {
        self.value.as_deref()
    }

    pub fn parse<T: FromStr>(&self) -> Option<Result<T, <T as FromStr>::Err>> {
        self.value.as_deref().map(|s| s.parse::<T>())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn whitespace_end_index() {
        let nowhite = "Key Value";
        let tab = "\tKey Value";
        let space = " Key Value";
        let twospace = "  Key Value";
        let mixed = "\t Key Value";

        assert_eq!(Value::whitespace_end_index(nowhite), 0);
        assert_eq!(Value::whitespace_end_index(tab), 1);
        assert_eq!(Value::whitespace_end_index(space), 1);
        assert_eq!(Value::whitespace_end_index(twospace), 2);
        assert_eq!(Value::whitespace_end_index(mixed), 2);
    }

    #[test]
    fn from_str() {
        let empty = "";
        assert_eq!(Value::from_str(empty).unwrap(), None);

        let noindent = "Key Value";
        let noindent_novalue = "Key";

        assert_eq!(
            Value::from_str(noindent).unwrap().unwrap(),
            Value::new(Indent::Empty, "Key", Some("Value".into()))
        );
        assert_eq!(
            Value::from_str(noindent_novalue).unwrap().unwrap(),
            Value::new(Indent::Empty, "Key", None)
        );

        let indent = "\tKey Value";
        let indent_novalue = "\tKey";

        assert_eq!(
            Value::from_str(indent).unwrap().unwrap(),
            Value::new(Indent::Tabs(1), "Key", Some("Value".into()))
        );
        assert_eq!(
            Value::from_str(indent_novalue).unwrap().unwrap(),
            Value::new(Indent::Tabs(1), "Key", None)
        );

        let mixed = " \tKey Value";
        assert_eq!(
            Value::from_str(mixed).unwrap_err(),
            ParseErrorKind::MixedIndent
        );
    }
}
