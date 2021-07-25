mod error;
mod indent;
mod value;

use std::str::FromStr;

pub use error::Error;
use error::ErrorKind;
use indent::Indent;
pub use value::Value;

pub struct Confindent {
    children: Vec<Value>,
}

impl Confindent {
    fn push(&mut self, value: Value) -> Result<(), ErrorKind> {
        match value.indent {
            Indent::Empty => self.children.push(value),
            Indent::Tabs(tabsize) => {
                let last = match self.children.last_mut() {
                    Some(val) => val,
                    None => return Err(ErrorKind::StartedIndented),
                };

				loop {
					match last.indent {
						Indent::Empty => last.children.push(value),
						Indent::Spaces(_) => return Err(ErrorKind::TabsWithSpaces),
						Indent::Tabs(level) => {
							
						}
					}
				}
            }
        }
    }
}

impl FromStr for Confindent {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut ret = Self { children: vec![] };
        let mut lines = s.lines().enumerate();
		let add_ln = |e: ErrorKind, ln: usize| -> Error {
			Error {
				line: ln,
				kind: e
			}
		};

        for (line_number, line) in lines {
            let value_op = Value::parse(line).map_err(|e| add_ln(e, line_number))?;

            let value = match value_op {
                Some(v) => ret.push(v).map_err(|e|),
                None => continue,
            };
        }

        Ok(ret)
    }
}
