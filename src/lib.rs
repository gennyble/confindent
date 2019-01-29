use std::collections::HashMap;
use std::str::FromStr;
use std::string::ParseError;

type ConfHash = HashMap<String, ConfSection>;

pub struct Confindent {
    sections: ConfHash,
}

impl Confindent {
    pub fn new() -> Self {
        Confindent {
            sections: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&ConfSection> {
        self.sections.get(key)
    }

    //TODO: implement
    pub fn from_file() {}

    //TOOD: implement
    pub fn from_buffer() {}

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

#[derive(Debug, PartialEq)]
pub struct ConfSection {
    value: ConfItem,
    indent_level: u8,
    children: ConfHash,
}

impl ConfSection {
    pub fn new(value: ConfItem, indent_level: u8, children: ConfHash) -> Self {
        ConfSection {
            value,
            indent_level,
            children,
        }
    }

    pub fn get(&self, key: &str) -> Option<&ConfSection> {
        self.children.get(key)
    }

    pub fn value(&self) -> &ConfItem {
        &self.value
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

#[derive(Debug, PartialEq)]
pub enum ConfItem {
    Empty,
    Text(String),
}

impl ConfItem {
    pub fn parse(s: &str) -> Self {
        ConfItem::Text(s.to_owned())
    }

    pub fn value<T: FromStr>(&self) -> Option<T> {
        match *self {
            ConfItem::Empty => None,
            ConfItem::Text(ref s) => s.parse().ok(),
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
    fn parse_config_early_api() {
        let config_string = "Host example.com\n\tUsername user\n\tPassword pass\nIdle 600";
        let config = Confindent::from_str(config_string).expect("Failed to parse config");

        let host_section = config.get("Host").expect("No Host in config");
        let hostname = host_section.value();
        let username = match host_section.get("Username") {
            Some(section) => section.value(),
            None => panic!(),
        };
        let password = match host_section.get("Password") {
            Some(section) => section.value(),
            None => panic!(),
        };

        let idle = match config.get("Idle") {
            Some(section) => section.value(),
            None => panic!(),
        };

        assert_eq!(hostname.value(), Some("example.com".to_string()));
        assert_eq!(username.value(), Some("user".to_string()));
        assert_eq!(password.value(), Some("pass".to_string()));
        assert_eq!(idle.value(), Some("600".to_string()));
    }
}
