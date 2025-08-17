use std::io::{Read, Seek};

use aira_tiff::{
    metadata::{Layout, Sample},
    Compression, Interpretation, SampleFormat, SubfileType,
};
use claims::*;

mod utils;

#[test]
fn decode_metadata() {
    let file = assert_ok!(std::fs::File::open("tests/images/tiled-rect-rgb-u8.tif"));
    let mut reader = std::io::BufReader::new(file);
    let metadata = utils::get_the_only_one_directory(&mut reader);

    assert_eq!(metadata.dimensions, (490, 367));
    assert_eq!(metadata.interpretation, Interpretation::RGB);
    assert_eq!(
        metadata.layout,
        Layout::Tiles {
            width: 32,
            length: 128,
        }
    );
    assert_eq!(metadata.compression, Compression::NONE);
    assert_eq!(metadata.subfile_type, SubfileType::default());
    assert_eq!(
        metadata.samples(),
        [Sample::new(SampleFormat::UNSIGNED, 8); 3]
    );

    assert_none!(metadata.artist());
    assert_none!(metadata.copyright());
    assert_none!(metadata.host_computer());
    assert_none!(metadata.description());
    assert_none!(metadata.software());

    assert_eq!(metadata.chunk_size(), (32, 128));
    assert_eq!(metadata.chunks_count(), 48);

    let mut buffer = Vec::<u8>::new();
    for chunk in metadata.chunks() {
        buffer.resize(chunk.byte_count as usize, 0u8);

        assert_ok!(reader.seek(std::io::SeekFrom::Start(chunk.offset)));
        assert_ok!(reader.read_exact(&mut buffer));
    }
}
