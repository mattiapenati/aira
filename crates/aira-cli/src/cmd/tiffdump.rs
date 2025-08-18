use std::{collections::HashSet, path::PathBuf};

use aira::tiff;
use anyhow::{ensure, Context};

pub struct TiffDump;

impl From<TiffDump> for clap::Command {
    fn from(_: TiffDump) -> Self {
        clap::Command::new(TiffDump::ID)
            .about("Display directory information from TIFF files")
            .arg(
                clap::Arg::new("json")
                    .long("json")
                    .help("The output is formatted as a JSON string")
                    .action(clap::ArgAction::SetTrue),
            )
            .arg(
                clap::Arg::new("items")
                    .short('m')
                    .long("max")
                    .help("Change the number of indirect data items that are printed")
                    .default_value("24")
                    .value_parser(clap::value_parser!(usize)),
            )
            .arg(
                clap::Arg::new("files")
                    .help("The list of files to be inspected")
                    .action(clap::ArgAction::Append)
                    .value_parser(clap::value_parser!(PathBuf))
                    .required(true),
            )
    }
}

macro_rules! print_string {
    ($string:ident [ .. $max:expr]) => {{
        if {
            let string = $string.chars().take($max).collect::<String>();
            print!("{}", crate::utils::JsonString(&string));
            string.len()
        } < $string.len()
        {
            print!("...");
        }
    }};
}

macro_rules! print_bytes {
    ($bytes:ident [ .. $max:expr]) => {{
        {
            let mut bytes = $bytes.iter().take($max);
            if let Some(first) = bytes.next() {
                print!("0x{first:02x}");
                for byte in bytes {
                    print!(" 0x{byte:02x}")
                }
            }
        }
        if $bytes.len() > $max {
            print!(" ...");
        }
    }};
}

macro_rules! print_values {
    ($values:ident [ .. $max:expr]) => {{
        {
            let mut values = $values.iter().take($max);
            if let Some(first) = values.next() {
                print!("{first}");
                for value in values {
                    print!(" {value}")
                }
            }
        }
        if $values.len() > $max {
            print!(" ...");
        }
    }};
}

macro_rules! print_json_values {
    ($writer:ident . $name:ident ( $values:ident ) ) => {{
        $writer.start_array()?;
        for value in &$values {
            $writer.$name(*value)?;
        }
        $writer.end_array()?;
    }};
}

macro_rules! print_ratio {
    ($values:ident [ .. $max:expr]) => {{
        {
            let mut values = $values.iter().take($max);
            if let Some(first) = values.next() {
                let first = (first.num as f64) / (first.den as f64);
                print!("{first}");
                for value in values {
                    let value = (value.num as f64) / (value.den as f64);
                    print!(" {value}")
                }
            }
        }
        if $values.len() > $max {
            print!(" ...");
        }
    }};
}

impl TiffDump {
    pub const ID: &'static str = "tiffdump";

    pub fn run(matches: &clap::ArgMatches) -> anyhow::Result<()> {
        let json = matches.get_flag("json");
        let maxitems = *matches
            .get_one::<usize>("items")
            .expect("Max items is required");
        let files = matches
            .get_many::<PathBuf>("files")
            .expect("Files are required")
            .cloned()
            .collect::<Vec<_>>();

        if json {
            dump_json(&files)
        } else {
            dump_terminal(&files, maxitems)
        }
    }
}

fn dump_json(files: &[PathBuf]) -> anyhow::Result<()> {
    let mut writer = crate::utils::JsonWriter::new(std::io::stdout());
    let multiple_files = files.len() > 1;

    if multiple_files {
        writer.start_array()?;
    }

    for path in files {
        let file = std::fs::File::open(path)
            .with_context(|| format!("Failed to open {}", path.display()))?;
        writer.start_object()?;

        if multiple_files {
            writer.write_key("path")?;
            writer.write_str(&path.display().to_string())?;
        }

        let reader = std::io::BufReader::new(file);
        let mut decoder = tiff::Decoder::new(reader)?;

        let byteorder = decoder.byteorder();
        let version = decoder.version();

        writer.write_key("byteorder")?;
        {
            writer.start_object()?;
            writer.write_key("id")?;
            writer.write_u16(byteorder as u16)?;
            writer.write_key("name")?;
            writer.write_str(match byteorder {
                tiff::ByteOrder::BigEndian => "big-endian",
                tiff::ByteOrder::LittleEndian => "little-endian",
            })?;
            writer.end_object()?;
        }
        writer.write_key("version")?;
        {
            writer.start_object()?;
            writer.write_key("id")?;
            writer.write_u16(version as u16)?;
            writer.write_key("name")?;
            writer.write_str(match version {
                tiff::Version::Classic => "ClassicTIFF",
                tiff::Version::BigTiff => "BigTIFF",
            })?;
            writer.end_object()?;
        }

        writer.write_key("directories")?;

        let mut directories = decoder.directories();
        let mut visited_offsets = HashSet::new();

        writer.start_array()?;
        while let Some(directory) = directories.next_directory()? {
            ensure!(
                visited_offsets.insert(directory.offset),
                "Cycle detected in chaining of TIFF directories"
            );

            writer.start_object()?;

            writer.write_key("offset")?;
            writer.write_u64(directory.offset)?;
            writer.write_key("next")?;
            writer.write_u64(directory.next_offset)?;

            writer.write_key("entries")?;
            writer.start_array()?;
            let mut entries = directory.entries();
            while let Some(entry) = entries.next_entry()? {
                writer.start_object()?;

                writer.write_key("tag")?;
                {
                    writer.start_object()?;
                    writer.write_key("id")?;
                    writer.write_u16(entry.tag.0)?;
                    writer.write_key("name")?;
                    writer.write_str(entry.tag.name())?;
                    writer.end_object()?;
                }
                writer.write_key("dtype")?;
                {
                    writer.start_object()?;
                    writer.write_key("id")?;
                    writer.write_u16(entry.dtype as u16)?;
                    writer.write_key("name")?;
                    writer.write_str(entry.dtype.name())?;
                    writer.end_object()?;
                }
                writer.write_key("count")?;
                writer.write_u64(entry.count)?;

                writer.write_key("value")?;
                match tiff::Entry::from_decoder(entry)? {
                    tiff::Entry::Ascii(string) => writer.write_str(&string)?,
                    tiff::Entry::Bytes(values) => print_json_values!(writer.write_u8(values)),
                    tiff::Entry::U8(values) => print_json_values!(writer.write_u8(values)),
                    tiff::Entry::U16(values) => print_json_values!(writer.write_u16(values)),
                    tiff::Entry::U32(values) => print_json_values!(writer.write_u32(values)),
                    tiff::Entry::U64(values) => print_json_values!(writer.write_u64(values)),
                    tiff::Entry::I8(values) => print_json_values!(writer.write_i8(values)),
                    tiff::Entry::I16(values) => print_json_values!(writer.write_i16(values)),
                    tiff::Entry::I32(values) => print_json_values!(writer.write_i32(values)),
                    tiff::Entry::I64(values) => print_json_values!(writer.write_i64(values)),
                    tiff::Entry::F32(values) => print_json_values!(writer.write_f32(values)),
                    tiff::Entry::F64(values) => print_json_values!(writer.write_f64(values)),
                    _ => {}
                }
                writer.end_object()?;
            }
            writer.end_array()?;

            writer.end_object()?;
        }
        writer.end_array()?;

        writer.end_object()?;
    }

    if multiple_files {
        writer.end_array()?;
    }

    Ok(())
}

fn dump_terminal(files: &[PathBuf], maxitems: usize) -> anyhow::Result<()> {
    let multiple_files = files.len() > 1;

    for (index, path) in files.iter().enumerate() {
        let file = std::fs::File::open(path)
            .with_context(|| format!("Failed to open {}", path.display()))?;

        if index > 0 {
            println!();
        }

        if multiple_files {
            println!("{}", path.display());
        }

        let reader = std::io::BufReader::new(file);
        let mut decoder = tiff::Decoder::new(reader)?;

        let byteorder = decoder.byteorder();
        let version = decoder.version();
        println!(
            "Magic: 0x{:04x} <{}-endian> Version: 0x{:02x} <{}>",
            byteorder as u16,
            match byteorder {
                tiff::ByteOrder::BigEndian => "big",
                tiff::ByteOrder::LittleEndian => "little",
            },
            version as u8,
            match version {
                tiff::Version::Classic => "ClassicTIFF",
                tiff::Version::BigTiff => "BigTIFF",
            }
        );

        let mut directories = decoder.directories();
        let mut visited_offsets = HashSet::new();
        let mut directory_index = 0;
        while let Some(directory) = directories.next_directory()? {
            ensure!(
                visited_offsets.insert(directory.offset),
                "Cycle detected in chaining of TIFF directories"
            );

            if directory_index > 0 {
                println!();
            }

            println!(
                "Directory {directory_index}: offset {offset} (0x{offset:x}) \
                    next {next} (0x{next:x})",
                offset = directory.offset,
                next = directory.next_offset,
            );

            let mut entries = directory.entries();
            while let Some(entry) = entries.next_entry()? {
                print!("{:?} {:?} {}<", entry.tag, entry.dtype, entry.count);
                match tiff::Entry::from_decoder(entry)? {
                    tiff::Entry::Ascii(string) => print_string!(string[..maxitems]),
                    tiff::Entry::Bytes(bytes) => print_bytes!(bytes[..maxitems]),
                    tiff::Entry::U8(values) => print_values!(values[..maxitems]),
                    tiff::Entry::U16(values) => print_values!(values[..maxitems]),
                    tiff::Entry::U32(values) => print_values!(values[..maxitems]),
                    tiff::Entry::U64(values) => print_values!(values[..maxitems]),
                    tiff::Entry::I8(values) => print_values!(values[..maxitems]),
                    tiff::Entry::I16(values) => print_values!(values[..maxitems]),
                    tiff::Entry::I32(values) => print_values!(values[..maxitems]),
                    tiff::Entry::I64(values) => print_values!(values[..maxitems]),
                    tiff::Entry::F32(values) => print_values!(values[..maxitems]),
                    tiff::Entry::F64(values) => print_values!(values[..maxitems]),
                    tiff::Entry::Ratio(values) => print_ratio!(values[..maxitems]),
                    tiff::Entry::SignedRatio(values) => print_ratio!(values[..maxitems]),
                }
                println!(">");
            }

            directory_index += 1;
        }
    }

    Ok(())
}
