# Confindent
[![Crates](https://flat.badgen.net/crates/v/confindent)][crate]
[![Downloads](https://flat.badgen.net/crates/d/confindent)][crate]

[crate]: https://crates.io/crates/confindent

**Conf**iguration by **indent**ation. Read the [spec](spec.md) inspired by
the format of the ssh client configuration commonly found on Linux machines
at `~/.ssh/config`.

#### Read config from a file
```rust
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
```
