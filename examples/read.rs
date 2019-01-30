extern crate confindent;
use confindent::{ConfParent, Confindent};

fn main() {
    //Read example.conf from the examples directory
    let conf = Confindent::from_file("examples/example.conf").unwrap();

    //Get the host section
    let host = conf.child("Host").unwrap();
    //Get the value of the host section
    let hostname: String = host.get().unwrap();
    //Get the value of the username subsection
    let username: String = host.child_value("Username").unwrap();
    //Get the value of the password subsection
    let password: String = host.child_value("Password").unwrap();

    println!("ssh {}@{} -p {}", username, hostname, password);
}
