use confindent::Confindent;

fn main() {
	let conf: Confindent = "Pet Dog\n\tName Brady\n\tAge 10".parse().unwrap();
	let pet = conf.child("Pet").unwrap();
	let name = pet.child_value("Name").unwrap();
	let age: usize = pet.child_parse("Age").unwrap();

	let word = match pet.value() {
		Some("Dog") => "pupper",
		Some("Cat") => "kitty",
		_ => panic!(),
	};

	if age > 9 {
		println!("{}! {} is an old {}.", age, name, word);
	} else {
		println!("Only {}! {} is a good, young {}.", age, name, word);
	}
}
