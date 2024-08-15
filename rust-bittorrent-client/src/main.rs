fn main() {
    let bytestring = std::fs::read("./archlinux-2022.11.01-x86_64.iso.torrent").unwrap();

    let decoded = bencoding::decode(&bytestring).unwrap();
    println!("{:#?}", decoded);
    let encoded = bencoding::encode(decoded);
    println!("{:#?}", encoded);

    assert_eq!(bytestring, encoded);
}
