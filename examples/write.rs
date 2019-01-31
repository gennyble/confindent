extern crate confindent;
use confindent::{ConfParent, Confindent};

fn main() {
    let mut conf = Confindent::new();
    conf.create("Host", "example.net").create("Idle", "3600");
    conf.child_mut("Host")
        .unwrap()
        .create("Username", "gerald")
        .create("Password", "qwerty");

    conf.to_file("example_write.conf").unwrap();
}
