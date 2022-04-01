# Confindent

[![Crates](https://flat.badgen.net/crates/v/confindent)][crate]
[![Docs](https://docs.rs/confindent/badge.svg)](https://docs.rs/confindent)
[![Downloads](https://flat.badgen.net/crates/d/confindent)][crate]

[crate]: https://crates.io/crates/confindent

**Conf**iguration by **indent**ation. Read the [spec][spec] inspired by
the format of the ssh client configuration commonly found on Linux machines
at `~/.ssh/config`.

[spec]: https://github.com/gennyble/confindent/blob/main/spec.md

## Example, short and sweet
```rust
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
```

## Quickstart!

#### The format, briefly. [here's the very verbose spec](https://github.com/gennyble/confindent/blob/main/spec.md)
It's a kind of tree, key-value thing. Lines are key-value pairs, the value
starting at the first space after the indent. You can add a child to a value
by indenting it with spaces or tabs. Indent the same amount to add another
child to that same value. Indent more than you did initially to add a
grandchild. Don't mix spaces and tabs. Like this!

```ignore
Root this is the root
	Child I'm a child!
	Child You can have multiple children with the same keys!
		Grandchild I'm a grandchild!
```

#### Using the crate, quickly! [also, here are the docs again](https://docs.rs/confindent)

Open and parse a file with [`Confindent::from_file`][ff]. Pass it a path. It returns
a `Result<Confindent, ParseError>`.

Get a direct child with the [`child(key)`][child] function. Key needs to be able
to turn into a `&str`. This returns an `Option<&Value>`. [`Value`][value] is the main data-storing
struct. You can get multiple Value of the same name with [`children(key)`][children], which
returns a `Vec<&Value>`.

You can get a `Value`'s value with [`value()`][fn-value]. It returns an `Option<&str>`.

Want to parse a possible value into a different type, `T`? Instead of `value()` use
[`parse()`][parse]. It returns `Result<T, ValueParseError<T>>`. That type
may look weird and that's because it is. [`ValueParseError`][vperror] is an enum
that can be `NoValue` or `ParseError(error)` where `error` is the error part of the
Result that `T::FromStr` returns.

Don't want to call `child(key)` and then `value()` or `parse()`? You can use
[`child_value(key)`][childvalue] and [`child_parse(key)`][childparse] to do both of those
at once. Both of these functions return what `value()` and `parse()` normally return,
respectively.

[ff]: https://docs.rs/confindent/latest/confindent/struct.Confindent.html#method.from_file
[child]: https://docs.rs/confindent/latest/confindent/struct.Value.html#method.child
[children]: https://docs.rs/confindent/latest/confindent/struct.Value.html#method.children
[value]: https://docs.rs/confindent/latest/confindent/struct.Value.html
[fn-value]: https://docs.rs/confindent/latest/confindent/struct.Value.html#method.value
[parse]: https://docs.rs/confindent/latest/confindent/struct.Value.html#method.parse
[vperror]: https://docs.rs/confindent/latest/confindent/enum.ValueParseError.html
[childvalue]: https://docs.rs/confindent/latest/confindent/struct.Value.html#method.child_value
[childparse]: https://docs.rs/confindent/latest/confindent/struct.Value.html#method.child_parse
