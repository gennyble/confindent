use std::str::FromStr;

use confindent::{Confindent, Node};

fn main() {
	let mut conf = Confindent::from_str(&INITIAL).unwrap();
	let child = conf.child_mut("Key").unwrap();

	child.push_entry(("Child2", 2048));

	let out = conf.to_string();

	print!("\"{out}\"")
}

#[rustfmt::skip]
const INITIAL: &'static str =
r#"Entry
	Link /relative/link"#;
