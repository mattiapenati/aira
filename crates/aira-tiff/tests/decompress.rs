use std::io::Read;

use aira_tiff::{compression::DecompressReader, Compression, Metadata};
use claims::*;

mod utils;

fn try_decompress_all_chunks<R>(metadata: Metadata, reader: &mut R)
where
    R: std::io::Read + std::io::Seek,
{
    let mut buffer = Vec::<u8>::new();
    for chunk in metadata.chunks() {
        assert_ok!(reader.seek(std::io::SeekFrom::Start(chunk.offset)));
        let chunk_reader = reader.take(chunk.byte_count);
        let mut chunk_reader =
            assert_ok!(DecompressReader::new(chunk_reader, metadata.compression));

        assert_ok!(chunk_reader.read_to_end(&mut buffer));
    }
}

#[test]
fn decompress_none() {
    let file = assert_ok!(std::fs::File::open("tests/images/tiled-rect-rgb-u8.tif"));
    let mut reader = std::io::BufReader::new(file);
    let metadata = utils::get_the_only_one_directory(&mut reader);

    assert_eq!(metadata.compression, Compression::NONE);
    try_decompress_all_chunks(metadata, &mut reader);
}

#[test]
fn decompress_packbits() {
    let file = assert_ok!(std::fs::File::open(
        "tests/images/minisblack-2c-8b-alpha.tiff"
    ));
    let mut reader = std::io::BufReader::new(file);
    let metadata = utils::get_the_only_one_directory(&mut reader);

    assert_eq!(metadata.compression, Compression::PACKBITS);
    try_decompress_all_chunks(metadata, &mut reader);
}

#[cfg(feature = "deflate")]
#[test]
fn decompress_deflate() {
    let file = assert_ok!(std::fs::File::open("tests/images/random-fp16.tiff"));
    let mut reader = std::io::BufReader::new(file);
    let metadata = utils::get_the_only_one_directory(&mut reader);

    assert_eq!(metadata.compression, Compression::DEFLATE);
    try_decompress_all_chunks(metadata, &mut reader);
}
