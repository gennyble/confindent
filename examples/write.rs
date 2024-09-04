use std::str::FromStr;

use confindent::Confindent;

fn main() {
	let mut conf = Confindent::from_str(&INITIAL).unwrap();
	let child = conf.child_mut("Key").unwrap();

	for ch in &child.children {
		println!("{ch:?}");
	}

	child.push_comment("This is automatically written");
	child.push_value(("Child2", 2048));

	let out = conf.to_string();

	print!("\"{out}\"")
}

#[rustfmt::skip]
const INITIAL: &'static str =
r#"Key value
	Child value

Key2 value"#;
