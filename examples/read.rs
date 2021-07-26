use confindent::Confindent;

fn main() {
	let conf = Confindent::from_file("examples/example.conf").unwrap();

	let host = conf.child("Host").unwrap();
	let hostname = host.value().unwrap();

	for user in host.children("Username") {
		println!(
			"ssh {}@{} -p {}",
			user.value().unwrap(),
			hostname,
			user.child_value("Password").unwrap()
		);
	}
}
