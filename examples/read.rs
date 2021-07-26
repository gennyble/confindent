use confindent::Confindent;

fn main() {
    //Read example.conf from the examples directory
    let conf = Confindent::from_file("examples/example.conf").unwrap();

    let host = conf.child("Host").unwrap();
    let hostname = host.value().unwrap();

    let users = host.children("Username");

    for user in users {
        println!(
            "ssh {}@{} -p {}",
            user.value().unwrap(),
            hostname,
            user.child_value("Password").unwrap()
        );
    }
}
