use confindent::{get, let_get, Confindent, Node};

fn main() {
	let conf = Confindent::from_file("examples/songinfo.conf").unwrap();
	let song = conf.child("Song").unwrap();
	//let length: usize = song.child_parse("Length").unwrap();
	let_get!(
		song,
		clone artist = "Artist",
		length: usize = "Length",
		bitrate: usize = "Bitrate",
	);

	/*println!(
		"Now playing {} by {} [{}:{} {}kbps]",
		song.value().unwrap(),
		artist,
		length / 60, //minutes
		length % 60, //seconds
		bitrate
	);*/
}
