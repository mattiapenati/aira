use std::io::{Read, Seek};

use aira_tiff::{metadata::Layout, Decoder, Metadata};
use claims::*;

#[test]
fn decode_metadata() {
    let file = assert_ok!(std::fs::File::open("tests/images/tiled-rect-rgb-u8.tif"));
    let mut reader = std::io::BufReader::new(file);

    let metadata = {
        let mut decoder = assert_ok!(Decoder::new(&mut reader));
        let mut directories = decoder.directories();
        let directory = assert_some!(assert_ok!(directories.next_directory()));
        let metadata = assert_ok!(Metadata::from_decoder(directory));
        assert_none!(assert_ok!(directories.next_directory()));
        metadata
    };

    assert_eq!(metadata.dimensions, (490, 367));
    assert_eq!(
        metadata.layout,
        Layout::Tiles {
            width: 32,
            length: 128,
        }
    );
    assert_eq!(metadata.chunk_size(), (32, 128));
    assert_eq!(metadata.chunks_count(), 48);

    let mut buffer = Vec::<u8>::new();
    for chunk in metadata.chunks() {
        buffer.resize(chunk.byte_count as usize, 0u8);

        assert_ok!(reader.seek(std::io::SeekFrom::Start(chunk.offset)));
        assert_ok!(reader.read_exact(&mut buffer));
    }
}
