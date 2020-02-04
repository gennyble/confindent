///! Confindent is a library for **conf**iguration by **indent**ation.
///!
///! This tiny library provides a quick and easy way to read and write configuration.
///!
///! Features:
///! * no dependancies
///! * parse [from file](struct.Confindent.html#method.from_file) or [from string](struct.Confindent.html#impl-FromStr)
///! * create conf files with an intuitive [builder api](trait.ConfParent.html#method.child_mut)
///! * write [to file](struct.Confindent.html#method.to_string) or [to string](struct.Confindent.html#impl-Into)
use std::fmt;
use std::fs;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::Path;
use std::str::FromStr;
use std::string::ParseError;

type ConfVec = Vec<ConfSection>;

/// Structure for Reading/Writing configuration
#[derive(Debug, PartialEq)]
pub struct Confindent {
    sections: ConfVec,
}

impl Confindent {
    /// Create an empty configuration
    pub fn new() -> Self {
        Confindent {
            sections: Vec::new(),
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

    fn add_section(&mut self, cs: ConfSection) {
        if self.sections.is_empty() || cs.indent_level == 0 {
            self.sections.push(cs);
            return;
        }

        let iter = self.sections.iter_mut().rev();

        for sec in iter {
            if (*sec).indent_level == cs.indent_level - 1 {
                (*sec).children.push(cs);
                return;
            }
        }

        self.sections.push(cs);
    }

    fn get_sections_by_name(&self, key: &str) -> Option<Vec<&ConfSection>> {
        let mut ret: Vec<&ConfSection> = Vec::new();

        for sec in &self.sections {
            if sec.key == key {
                ret.push(sec);
            }
        }

        if ret.is_empty() {
            None
        } else {
            Some(ret)
        }
    }

    fn get_sections_by_name_mut(&mut self, key: &str) -> Option<Vec<&mut ConfSection>> {
        let mut ret: Vec<&mut ConfSection> = Vec::new();

        for sec in &mut self.sections {
            if sec.key == key {
                ret.push(sec);
            }
        }

        if ret.is_empty() {
            None
        } else {
            Some(ret)
        }
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
                Some(v) => ret.add_section(v),
                None => continue,
            }
        }

        Ok(ret)
    }
}

impl ConfParent for Confindent {
    fn get_children<T: Into<String>>(&self, key: T) -> Option<Vec<&ConfSection>> {
        self.get_sections_by_name(&key.into())
    }

    fn get_children_mut<T: Into<String>>(&mut self, key: T) -> Option<Vec<&mut ConfSection>> {
        self.get_sections_by_name_mut(&key.into())
    }

    fn create_child<T: Into<String>>(&mut self, key: T, value: T) -> &mut Self {
        let sec = ConfSection::new(key.into(), ConfItem::parse(&value.into()), 0, Vec::new());
        self.sections.push(sec);

        self
    }
}

impl Into<String> for Confindent {
    fn into(self) -> String {
        let mut ret = String::new();

        for child in self.sections {
            ret.push_str(&format!("\n{}", child.into_string()));
        }

        ret.trim().to_owned()
    }
}

#[derive(Debug, PartialEq)]
pub struct ConfSection {
    key: String,
    value: ConfItem,
    indent_level: u8,
    children: ConfVec,
}

impl ConfSection {
    fn new(key: String, value: ConfItem, indent_level: u8, children: ConfVec) -> Self {
        ConfSection {
            key,
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

    fn into_string(self) -> String {
        let mut ret = format!("{} {}", self.key, self.value);

        for child in self.children {
            let child_str = format!("\n\t{}", child.into_string().replace('\n', "\n\t"));
            ret.push_str(&child_str);
        }

        ret
    }

    fn parse(s: &str) -> Option<Self> {
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

        Some(Self::new(key, value, indent_level, Vec::new()))
    }

    fn get_sections_by_name(&self, key: &str) -> Option<Vec<&ConfSection>> {
        let mut ret: Vec<&ConfSection> = Vec::new();

        for sec in &self.children {
            if sec.key == key {
                ret.push(sec);
            }
        }

        if ret.is_empty() {
            None
        } else {
            Some(ret)
        }
    }

    fn get_sections_by_name_mut(&mut self, key: &str) -> Option<Vec<&mut ConfSection>> {
        let mut ret: Vec<&mut ConfSection> = Vec::new();

        for sec in &mut self.children {
            if sec.key == key {
                ret.push(sec);
            }
        }

        if ret.is_empty() {
            None
        } else {
            Some(ret)
        }
    }
}

impl ConfParent for ConfSection {
    fn get_children<T: Into<String>>(&self, key: T) -> Option<Vec<&ConfSection>> {
        self.get_sections_by_name(&key.into())
    }

    fn get_children_mut<T: Into<String>>(&mut self, key: T) -> Option<Vec<&mut ConfSection>> {
        self.get_sections_by_name_mut(&key.into())
    }

    fn create_child<T: Into<String>>(&mut self, key: T, value: T) -> &mut Self {
        let sec = ConfSection::new(key.into(), ConfItem::parse(&value.into()), 0, Vec::new());
        self.children.push(sec);

        self
    }
}

#[derive(Debug, PartialEq)]
enum ConfItem {
    Empty,
    Text(String),
}

impl ConfItem {
    fn parse(s: &str) -> Self {
        ConfItem::Text(s.to_owned())
    }

    fn get<T: FromStr>(&self) -> Option<T> {
        match *self {
            ConfItem::Empty => None,
            ConfItem::Text(ref s) => s.parse().ok(),
        }
    }
}

/// Methods for configuration sections with children
pub trait ConfParent {
    /// Get a reference to the first child section with this name
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
    fn get_child<T: Into<String>>(&self, key: T) -> Option<&ConfSection> {
        let children_vec = self.get_children(key);

        match children_vec {
            Some(mut vec) => {
                if vec.len() > 0 {
                    Some(vec.remove(0))
                } else {
                    None
                }
            }
            None => None,
        }
    }

    /// Shorthand for [`get_child()`](#method.get_child)
    fn child<T: Into<String>>(&self, key: T) -> Option<&ConfSection> {
        self.get_child(key)
    }

    /// Get a Vec of children with this name
    ///
    /// ## Example
    /// ```
    /// use std::str::FromStr;
    /// use confindent::{Confindent, ConfParent};
    ///
    /// let conf_str = "Section value0\nSection value1";
    /// let conf = Confindent::from_str(conf_str).unwrap();
    /// let sections = conf.get_children("Section").unwrap();
    /// ```
    fn get_children<T: Into<String>>(&self, key: T) -> Option<Vec<&ConfSection>>;

    //// Shorthand for [`get_children()`](#method.get_children)
    fn children<T: Into<String>>(&self, key: T) -> Option<Vec<&ConfSection>> {
        self.get_children(key)
    }

    /// Get a mutable reference to the first child section with this name
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
    fn get_child_mut<T: Into<String>>(&mut self, key: T) -> Option<&mut ConfSection> {
        let children_vec = self.get_children_mut(key);

        match children_vec {
            Some(mut vec) => {
                if vec.len() > 0 {
                    Some(vec.remove(0))
                } else {
                    None
                }
            }
            None => None,
        }
    }

    /// Shorthand for [`get_child_mut()`](#method.get_child_mut)
    fn child_mut<T: Into<String>>(&mut self, key: T) -> Option<&mut ConfSection> {
        self.get_child_mut(key)
    }

    /// Get a Vec of mutable children with this name
    ///
    /// ## Example
    /// ```
    /// use std::str::FromStr;
    /// use confindent::{Confindent, ConfParent};
    ///
    /// let conf_str = "Section value0\nSection value1";
    /// let mut conf = Confindent::from_str(conf_str).unwrap();
    /// let mut sections = conf.get_children_mut("Section").unwrap();
    /// ```
    fn get_children_mut<T: Into<String>>(&mut self, key: T) -> Option<Vec<&mut ConfSection>>;

    //// Shorthand for [`get_children_mut()`](#method.get_children_mut)
    fn children_mut<T: Into<String>>(&mut self, key: T) -> Option<Vec<&mut ConfSection>> {
        self.get_children_mut(key)
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

    /// Get the value of the first child with this name
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
    fn parse_section_empty() {
        assert_eq!(ConfSection::parse(""), None);
    }

    #[test]
    fn parse_section_onlyindent() {
        assert_eq!(ConfSection::parse("\t"), None);
    }

    #[test]
    fn parse_section_noindent() {
        let test_line = "Key Value";
        let section = ConfSection::parse(test_line).unwrap();

        assert_eq!(section.key, "Key");
        assert_eq!(section.value, ConfItem::Text("Value".to_string()));
        assert_eq!(section.indent_level, 0);
        assert!(section.children.is_empty());
    }

    #[test]
    fn parse_section_indent() {
        let test_line = "\tKey Value";
        let section = ConfSection::parse(test_line).unwrap();

        assert_eq!(section.key, "Key");
        assert_eq!(section.value, ConfItem::Text("Value".to_string()));
        assert_eq!(section.indent_level, 1);
        assert!(section.children.is_empty());
    }

    #[test]
    fn get_config_vec() {
        let test_line = "Vec 1,2,3,4";
        let section = ConfSection::parse(test_line).unwrap();

        assert_eq!(section.get_vec::<u8>().unwrap(), vec![1, 2, 3, 4]);
    }

    #[test]
    fn parse_config_one_section() {
        let test_line = "Key Value";
        let config = Confindent::from_str(test_line).unwrap();

        let first_section = config.child("Key").unwrap();
        assert_eq!(first_section.value, ConfItem::Text("Value".to_string()));
        assert_eq!(first_section.indent_level, 0);
        assert!(first_section.children.is_empty());
    }

    #[test]
    fn parse_config_two_sections() {
        let test_line = "Key Value\nKey2 Value2";
        let config = Confindent::from_str(test_line).unwrap();

        let first_section = config.child("Key").unwrap();
        assert_eq!(first_section.value, ConfItem::Text("Value".to_string()));
        assert_eq!(first_section.indent_level, 0);
        assert!(first_section.children.is_empty());

        let second_section = config.child("Key2").unwrap();
        assert_eq!(second_section.value, ConfItem::Text("Value2".to_string()));
        assert_eq!(second_section.indent_level, 0);
        assert!(second_section.children.is_empty());
    }

    #[test]
    fn parse_config_two_sections_same_name() {
        let test_line = "Key Value\nKey Value2";
        let config = Confindent::from_str(test_line).unwrap();

        let mut keys_key = config.get_children("Key").unwrap();
        let second = keys_key.pop().unwrap();
        let first = keys_key.pop().unwrap();

        assert_eq!(first.key, "Key");
        assert_eq!(first.value, ConfItem::Text("Value".to_string()));
        assert_eq!(second.key, "Key");
        assert_eq!(second.value, ConfItem::Text("Value2".to_string()));
    }

    #[test]
    fn parse_config_nested_sections() {
        let test_line = "Key Value\n\tChild Value2";
        let config = Confindent::from_str(test_line).unwrap();

        let first_section = config.child("Key").unwrap();
        assert_eq!(first_section.value, ConfItem::Text("Value".to_string()));
        assert_eq!(first_section.indent_level, 0);
        assert_eq!(first_section.children.len(), 1);

        let second_section = first_section.child("Child").unwrap();
        assert_eq!(second_section.value, ConfItem::Text("Value2".to_string()));
        assert_eq!(second_section.indent_level, 1);
        assert!(second_section.children.is_empty());
    }

    #[test]
    fn parse_config_from_str() {
        let config_string = "Host example.com\n\tUsername user\n\tPassword pass\n\nIdle 600";
        let config = Confindent::from_str(config_string).expect("Failed to parse config");
        verify_full_parse(&config)
    }

    #[test]
    fn parse_config_from_file() {
        let config =
            Confindent::from_file("examples/example.conf").expect("Failed to parse config");
        verify_full_parse(&config)
    }

    fn verify_full_parse(config: &Confindent) {
        let host_section = config.child("Host").expect("No Host in config");
        let hostname = host_section.get();
        let username = match host_section.child("Username") {
            Some(section) => section.get(),
            None => panic!(),
        };
        let password = match host_section.child("Password") {
            Some(section) => section.get(),
            None => panic!(),
        };

        let idle = match config.child("Idle") {
            Some(section) => section.get(),
            None => panic!(),
        };

        assert_eq!(hostname, Some("example.com".to_string()));
        assert_eq!(username, Some("user".to_string()));
        assert_eq!(password, Some("pass".to_string()));
        assert_eq!(idle, Some("600".to_string()));
    }
}
