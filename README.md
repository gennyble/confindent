# Confindent
[![Crates](https://flat.badgen.net/crates/v/confindent)][crate]
[![Downloads](https://flat.badgen.net/crates/d/confindent)][crate]
[![License](https://flat.badgen.net/github/license/genuinebyte/confindent)][github]

[crate]: https://crates.io/crates/confindent
[github]: https://github.com/genuinebyte/confindent

**Conf**iguration by **indent**ation. Read the [spec](spec.md) inspired by
the format of the ssh client configuration commonly found on Linux machines
at `~/.ssh/config`.

#### Read configuration from a file
```rust
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

    //Result:
    //ssh user@example.com -p pass
}
```

#### Write cconfiguration to a file
```rust
extern crate confindent;
use confindent::{ConfParent, Confindent};

fn main() {
    let mut conf = Confindent::new();
    conf.create("Host", "example.net").create("Idle", "3600");
    conf.child_mut("Host")
        .unwrap()
        .create("Username", "gerald")
        .create("Password", "qwerty");
    
    conf.to_file("example.conf").unwrap();
    /*
    Yields the file `example.conf` with contents:
    
    Host example.net
    	Password qwerty
    	Username gerald
    Idle 3600
    */
}
```