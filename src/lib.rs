use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;
use std::str::FromStr;
use std::string::ParseError;

type ConfHash = HashMap<String, ConfSection>;

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

    ///Creates a new configuration from a file
    pub fn from_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let string = fs::read_to_string(path)?;
        Ok(Confindent::from_str(&string).expect("This should not happen"))
    }

    fn add_section(&mut self, key: String, cs: ConfSection) {
        if self.sections.is_empty() || cs.indent_level == 0 {
            self.sections.insert(key, cs);
            return;
        }

        let mut hashvec: Vec<(&String, &mut ConfSection)> = self.sections.iter_mut().collect();
        let iter = hashvec.iter_mut().rev();

        for (_, sec) in iter {
            if (**sec).indent_level == cs.indent_level - 1 {
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
        let sec = ConfSection::new(ConfItem::parse(&value.into()), 1, HashMap::new());
        self.sections.insert(key.into(), sec);

        self
    }
}

#[derive(Debug, PartialEq)]
pub struct ConfSection {
    value: ConfItem,
    indent_level: u8,
    children: ConfHash,
}

impl ConfSection {
    fn new(value: ConfItem, indent_level: u8, children: ConfHash) -> Self {
        ConfSection {
            value,
            indent_level,
            children,
        }
    }

    ///Set the value of this section
    pub fn set_value<T: Into<String>>(&mut self, value: T) -> &mut Self {
        self.value = ConfItem::parse(&value.into());

        self
    }

    ///Shorthand for [`set_value()`](#method.set_value)
    pub fn set<T: Into<String>>(&mut self, value: T) -> &mut Self {
        self.set_value(value)
    }

    ///Get the scalar value of this section
    pub fn get_value<T: FromStr>(&self) -> Option<T> {
        self.value.get()
    }

    ///Shorthand for [`get_value()`](#method.get_value)
    pub fn get<T: FromStr>(&self) -> Option<T> {
        self.get_value()
    }

    fn parse(s: &str) -> Option<(String, Self)> {
        if s.is_empty() || s.trim_start().is_empty() {
            return None;
        }

        let mut workable: &str = &s;

        let mut indent_level = 0;
        while workable.starts_with('\t') {
            indent_level += 1;
            workable = match workable.get(1..) {
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

#[derive(Debug, PartialEq)]
enum ConfItem {
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

pub trait ConfParent {
    ///Get a reference to a child section
    fn get_child<T: Into<String>>(&self, key: T) -> Option<&ConfSection>;

    ///Shorthand for [`get_child()`](#method.get_child)
    fn child<T: Into<String>>(&self, key: T) -> Option<&ConfSection> {
        self.get_child(key)
    }

    ///Get a mutable reference to a child section
    fn get_child_mut<T: Into<String>>(&mut self, key: T) -> Option<&mut ConfSection>;

    ///Shorthand for [`get_child_mut()`](#method.get_child_mut)
    fn child_mut<T: Into<String>>(&mut self, key: T) -> Option<&mut ConfSection> {
        self.get_child_mut(key)
    }

    ///Create a child
    fn create_child<T: Into<String>>(&mut self, key: T, value: T) -> &mut Self;

    ///Shorthand for [`create_child()`](#method.create_child)
    fn create<T: Into<String>>(&mut self, key: T, value: T) -> &mut Self {
        self.create_child(key, value)
    }

    ///Get the value of a child
    fn get_child_value<T: Into<String>, Y: FromStr>(&self, key: T) -> Option<Y> {
        match self.get_child(key) {
            None => None,
            Some(child) => child.get(),
        }
    }

    ///Shorthand for [`get_child_value()`](#mathod.get_child_value)
    fn child_value<T: Into<String>, Y: FromStr>(&self, key: T) -> Option<Y> {
        self.get_child_value(key)
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
        let (key, section) = ConfSection::parse(test_line).unwrap();

        assert_eq!(key, "Key");
        assert_eq!(section.value, ConfItem::Text("Value".to_string()));
        assert_eq!(section.indent_level, 0);
        assert!(section.children.is_empty());
    }

    #[test]
    fn parse_section_indent() {
        let test_line = "\tKey Value";
        let (key, section) = ConfSection::parse(test_line).unwrap();

        assert_eq!(key, "Key");
        assert_eq!(section.value, ConfItem::Text("Value".to_string()));
        assert_eq!(section.indent_level, 1);
        assert!(section.children.is_empty());
    }

    #[test]
    fn parse_config_one_section() {
        let test_line = "Key Value";
        let config = Confindent::from_str(test_line).unwrap();

        let first_section = config.sections.get("Key").unwrap();
        assert_eq!(first_section.value, ConfItem::Text("Value".to_string()));
        assert_eq!(first_section.indent_level, 0);
        assert!(first_section.children.is_empty());
    }

    #[test]
    fn parse_config_two_sections() {
        let test_line = "Key Value\nKey2 Value2";
        let config = Confindent::from_str(test_line).unwrap();

        let first_section = config.sections.get("Key").unwrap();
        assert_eq!(first_section.value, ConfItem::Text("Value".to_string()));
        assert_eq!(first_section.indent_level, 0);
        assert!(first_section.children.is_empty());

        let second_section = config.sections.get("Key2").unwrap();
        assert_eq!(second_section.value, ConfItem::Text("Value2".to_string()));
        assert_eq!(second_section.indent_level, 0);
        assert!(second_section.children.is_empty());
    }

    #[test]
    fn parse_config_nested_sections() {
        let test_line = "Key Value\n\tChild Value2";
        let config = Confindent::from_str(test_line).unwrap();

        let first_section = config.sections.get("Key").unwrap();
        assert_eq!(first_section.value, ConfItem::Text("Value".to_string()));
        assert_eq!(first_section.indent_level, 0);
        assert_eq!(first_section.children.len(), 1);

        let second_section = first_section.children.get("Child").unwrap();
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
