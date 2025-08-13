use aira_tiff::{decoder::Decoder, Metadata};
use claims::*;

pub fn get_the_only_one_directory<R>(reader: R) -> Metadata
where
    R: std::io::Read + std::io::Seek + std::fmt::Debug,
{
    let mut decoder = assert_ok!(Decoder::new(reader));
    let mut directories = decoder.directories();
    let directory = assert_some!(assert_ok!(directories.next_directory()));
    let metadata = assert_ok!(Metadata::from_decoder(directory));
    assert_none!(assert_ok!(directories.next_directory()));
    metadata
}
