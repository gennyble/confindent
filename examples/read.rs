use confindent::Confindent;

fn main() {
	let conf = Confindent::from_file("examples/songinfo.conf").unwrap();
	let song = conf.child("Song").unwrap();
	let length: usize = song.child_parse("Length").unwrap();

	println!(
		"Now playing {} by {} [{}:{} {}kbps]",
		song.value().unwrap(),
		song.child_value("Artist").unwrap(),
		length / 60, //minutes
		length % 60, //seconds
		song.child_value("Bitrate").unwrap()
	);
}
