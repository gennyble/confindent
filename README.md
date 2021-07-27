# Confindent
[![Crates](https://flat.badgen.net/crates/v/confindent)][crate]
[![Downloads](https://flat.badgen.net/crates/d/confindent)][crate]

[crate]: https://crates.io/crates/confindent

**Conf**iguration by **indent**ation. Read the [spec](spec.md) inspired by
the format of the ssh client configuration commonly found on Linux machines
at `~/.ssh/config`.

## Quickstart!

#### The format, briefly.
It's a kind of tree, key-value thing. Lines are key-value pairs, the value
starting at the first space after the indent. You can add a child to a value
by indenting it with spaces or tabs. Indent the same amount to add another
child to that same value. Indent more than you did initially to add a
grandchild. Don't mix spaces and tabs. Like this!
```
Root this is the root
	Child I'm a child!
	Child You can have multiple children with the same keys!
		Grandchild I'm a grandchild!
```

#### Using the crate, quickly! [also, here are the docs](https://docs.rs/confindent)

Open and parse a file with [`Confindent::from_file`][ff]. Pass it a path. It returns
a Result.

Get a direct child with the [`child(key)`][child] function. Key needs to be able
to turn into a &str. This returns an Option<&Value>. [`Value`][value] is the main data-storing
struct. You can get multiple Value of the same name with [`children(key)`][children], which
returns a Vec<&Value>.

You can get a Value's value with [`value()`][fn-value]. It returns an Option<&str>.

Want to parse a possible value into a different type, T? Instead of `value()` use
[`parse()`][parse]. It returns the same Result as if you tried to parse a string
into the value you want; the same as T:from_str. Remember how `value()`
returns an Option? If there is no value present, **`parse()` will do T::from_str("").**

Don't want to call `child(key)` and then `value()` or `parse()`? You can use
[`child_value(key)`][childvalue] and [`child_parse(key)`][childparse] to do both of those
at once. Remember how **`parse()` will give T::FromStr() an empty string** if there is no
value? Well, if no child is found on a call to `child_parse(key)`, it does the same thing.
This is the best way to go about not returning an `Option<Result<>>`, but
still providing the error if parsing fails. 

[ff]: https://docs.rs//confindent/2.0.0/confindent/struct.Confindent.html#method.from_file
[child]: https://docs.rs//confindent/2.0.0/confindent/struct.Value.html#method.child
[children]: https://docs.rs//confindent/2.0.0/confindent/struct.Value.html#method.children
[value]: https://docs.rs//confindent/2.0.0/confindent/struct.Value.html#method.value
[parse]: https://docs.rs//confindent/2.0.0/confindent/struct.Value.html#method.parse
[childvalue]: https://docs.rs//confindent/2.0.0/confindent/struct.Value.html#method.childvalue
[childparse]: https://docs.rs//confindent/2.0.0/confindent/struct.Value.html#method.childparse