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

//TODO: gen- Consider using $tt to preprocess tokens:
// https://palant.info/2023/04/17/processing-a-complex-syntax-with-rusts-declarative-macros/
/*#[macro_export]
macro_rules! let_get {
	($conf:ident / $varname:ident = $key:literal) => {
		let $varname = $conf.get($key).unwrap().to_owned();
	};

	($conf:ident / catch { $catch:tt }; $varname:ident = $key:literal) => {
		let Some($varname) = $conf.get($key).map(|v| v.to_owned()) else {
			$catch
		};
	};

	($conf:ident / $varname:ident: $vartype:ty = $key:literal) => {
		let $varname: $vartype = $conf.child_parse($key).unwrap();
	};

	($conf:ident / catch { $catch:tt }; $varname:ident: $vartype:ty = $key:literal) => {
		let Some($varname): $vartype = $conf.child_parse($key).ok().unwrap() else {
			$catch
		};
	};

	($conf:ident / $varname:ident$(: $vartype:ty)? = $key:literal, $($varnames:ident$(: $vartypes:ty)? = $keys:literal),+ $(,)?) => {
		   let_get!($conf / $varname$(: $vartype)? = $key);
		   let_get!($conf / $($varnames$(: $vartypes)? = $keys),+)
	   };

	($conf:ident / catch { $catch:tt }; $varname:ident$(: $vartype:ty)? = $key:literal, $($varnames:ident$(: $vartypes:ty)? = $keys:literal),+ $(,)?) => {
		let_get!($conf / catch { $catch }; $varname$(: $vartype)? = $key);
		let_get!($conf / catch { $catch }; $($varnames$(: $vartypes)? = $keys)+ $catch)
	};
}*/

#[macro_export]
macro_rules! let_get {
	($conf:ident, $($tokens:tt)*) => {
		confindent::let_get_inner!([] $conf, $($tokens)*)
	};
}

#[macro_export]
macro_rules! let_get_inner {
	// A simple let... the borrows
	// let key = conf.get("Value").unwrap();
	{[$($processed:tt)*] $conf:ident, $varname:ident = $key:literal, $($rest:tt)*} => {
		confindent::let_get_inner!{[
			$($processed)*
			let $varname = $conf.get($key).unwrap();
		] $conf, $($rest)*}
	};

	// A simple let... that clones
	// let key = conf.get("Value").unwrap().to_owned();
	{[$($processed:tt)*] $conf:ident, clone $varname:ident = $key:literal, $($rest:tt)*} => {
		confindent::let_get_inner!{[
			$($processed)*
			let $varname = $conf.child_owned($key).unwrap();
		] $conf, $($rest)*}
	};

	// A let... but parsed to some type
	// let key: usize = conf.child_parse("Value").unwrap();
	{[$($processed:tt)*] $conf:ident, $varname:ident: $vartype:ty = $key:literal, $($rest:tt)*} => {
		confindent::let_get_inner!{[
			$($processed)*
			let $varname: $vartype = $conf.child_parse($key).unwrap();
		] $conf, $($rest)*}
	};

	{[$($processed:tt)*] $conf:ident,} => {
		$($processed)*
	}
}
