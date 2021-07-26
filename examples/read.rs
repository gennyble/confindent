use confindent::Confindent;

fn main() {
	let conf = Confindent::from_file("examples/example.conf").unwrap();

	let host = conf.child("Host").unwrap();
	let hostname = host.value().unwrap();
	let port: u16 = host.child_parse("Port").unwrap();

	for user in host.children("Username") {
		println!(
			"ssh {}@{} -p {} -P {}",
			user.value().unwrap(),
			hostname,
			user.child_value("Password").unwrap(),
			port
		);
	}
}
