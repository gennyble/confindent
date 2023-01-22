# Confindent

<div align="center">

[![Crates](https://flat.badgen.net/crates/v/confindent)][crate]
[![Docs](https://docs.rs/confindent/badge.svg)](https://docs.rs/confindent)
[![Downloads](https://flat.badgen.net/crates/d/confindent)][crate]
![GitHub workflows badge](https://github.com/gennyble/confindent/actions/workflows/actions.yml/badge.svg)

</div>

[crate]: https://crates.io/crates/confindent

**Conf**iguration by **indent**ation. Like `~/.ssh/config`.

## Example, short and sweet
```rust
use confindent::Confindent;
use std::error::Error;

fn main() {
	let conf: Confindent = "User gennyble\n\tEmail gen@nyble.dev\n\tID 256".parse().unwrap();

	let user = conf.child("User").unwrap();
	let username = user.value().unwrap();
	let email = user.child_value("Email").unwrap();
	let id: usize = user.child_parse("ID").unwrap();

	println!("User {username}: {id} Contact: {email}");
}
```

## Quickstart!

#### The format, briefly.
It's a kind of tree, key-value thing. Lines are key-value pairs, the value starting at the first
space after the indent. You can add a child to a value by indenting it with spaces or tabs. Indent
the same amount to add another child to that same value. Indent more than you did initially to add
a grandchild. Don't mix spaces and tabs. Like this!

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

You can get a `Value`'s value with [`value()`][fn-value]. It returns an `Option<&str>`. Get an owned,
`Option<String>` with [`value_owned()`][valueowned]. If you want
to check that a `Value` has a direct  child but don't care about the value, use
[`has_child(key)`][haschild]. It returns `bool` for whether or not a child was found with that key.

Want to parse a possible value into a different type, `T`? Instead of `value()` use
[`parse()`][parse]. It returns `Result<T, ValueParseError<T>>`. That type
may look weird and that's because it is. [`ValueParseError`][vperror] is an enum
that can be `NoValue` or `ParseError(error)` where `error` is the error part of the
Result that `T::FromStr` returns.

Don't want to call `child(key)` and then `value()` or `parse()`? You can use
[`child_value(key)`][childvalue] and [`child_parse(key)`][childparse] to do both of those
at once. Both of these functions return what `value()` and `parse()` normally return,
respectively. There's also [`child_owned()`][childowned] which is like `value_owned()` wherein
it returns an `Option<String>` of a child's value.

[ff]: https://docs.rs/confindent/latest/confindent/struct.Confindent.html#method.from_file
[child]: https://docs.rs/confindent/latest/confindent/struct.Value.html#method.child
[children]: https://docs.rs/confindent/latest/confindent/struct.Value.html#method.children
[value]: https://docs.rs/confindent/latest/confindent/struct.Value.html
[fn-value]: https://docs.rs/confindent/latest/confindent/struct.Value.html#method.value
[valueowned]: https://docs.rs/confindent/latest/confindent/struct.Value.html#method.value_owned
[haschild]: https://docs.rs/confindent/latest/confindent/struct.Value.html#method.has_child
[parse]: https://docs.rs/confindent/latest/confindent/struct.Value.html#method.parse
[vperror]: https://docs.rs/confindent/latest/confindent/enum.ValueParseError.html
[childvalue]: https://docs.rs/confindent/latest/confindent/struct.Value.html#method.child_value
[childowned]: https://docs.rs/confindent/latest/confindent/struct.Value.html#method.child_owned
[childparse]: https://docs.rs/confindent/latest/confindent/struct.Value.html#method.child_parse
