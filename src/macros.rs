//FIXME: gen- use expect here instead of unwrap. or a match and panic!()
#[macro_export]
macro_rules! get {
	($conf:ident $keys:literal) => {
		$conf.get($keys).unwrap()
	};

	(clone $conf:ident $keys:literal) => {
		$conf.get($keys).unwrap().to_owned()
	};
}

#[macro_export]
macro_rules! let_get {
	($conf:ident / $varname:ident = $key:literal) => {
		let $varname = $conf.get($key).unwrap().to_owned();
	};

	($conf:ident / $varname:ident = $key:literal, $($varnames:ident = $keys:literal),+) => {
		let_get!($conf / $varname = $key);
		let_get!($conf / $($varnames = $keys),+)
	};
}
