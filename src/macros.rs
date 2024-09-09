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

	($conf:ident / $varname:ident: $vartype:ty = $key:literal) => {
		let $varname: $vartype = $conf.child_parse($key).unwrap();
	};

	($conf:ident / $varname:ident$(: $vartype:ty)? = $key:literal, $($varnames:ident$(: $vartypes:ty)? = $keys:literal),+) => {
		   let_get!($conf / $varname$(: $vartype)? = $key);
		   let_get!($conf / $($varnames$(: $vartypes)? = $keys),+)
	   };
}
