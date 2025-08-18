/// A writer that allows writing JSON data incrementally.
#[derive(Debug)]
pub struct JsonWriter<W> {
    writer: W,
    stack: Vec<StackItem>,
}

#[derive(Debug, Eq, PartialEq)]
enum StackItem {
    Root,
    ArrayItem { first: bool },
    ObjectItem { next: ObjectItemNext },
}

#[derive(Debug, Eq, PartialEq)]
enum ObjectItemNext {
    FirstKey,
    Key,
    Value,
}

impl<W> JsonWriter<W> {
    /// Creates a new [`JsonWriter`] that sends the output to the given writer.
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            stack: vec![StackItem::Root],
        }
    }
}

macro_rules! write_numeric {
    ($name:ident($value:ident: $ty:ty)) => {
        pub fn $name(&mut self, $value: $ty) -> std::io::Result<()> {
            self.start_value()?;
            self.writer.write_all($value.to_string().as_bytes())?;
            self.end_value()?;
            Ok(())
        }
    };
}

impl<W> JsonWriter<W>
where
    W: std::io::Write,
{
    /// Open a new array.
    pub fn start_array(&mut self) -> std::io::Result<()> {
        self.start_value()?;
        self.writer.write_all(b"[")?;
        self.stack.push(StackItem::ArrayItem { first: true });
        Ok(())
    }

    /// Close an array.
    pub fn end_array(&mut self) -> std::io::Result<()> {
        let last = self.stack.pop().expect("Unmatched array end");
        match last {
            StackItem::ArrayItem { .. } => {}
            _ => unreachable!("end_array called outside of an array"),
        }
        self.writer.write_all(b"]")?;
        self.end_value()?;
        Ok(())
    }

    /// Open a new object.
    pub fn start_object(&mut self) -> std::io::Result<()> {
        self.start_value()?;
        self.writer.write_all(b"{")?;
        self.stack.push(StackItem::ObjectItem {
            next: ObjectItemNext::FirstKey,
        });
        Ok(())
    }

    /// Close an object.
    pub fn end_object(&mut self) -> std::io::Result<()> {
        let last = self.stack.pop().expect("Unmatched object end");
        match last {
            StackItem::ObjectItem {
                next: ObjectItemNext::FirstKey | ObjectItemNext::Key,
            } => {}
            StackItem::ObjectItem { .. } => {
                unreachable!("end_object called with missing value");
            }
            _ => unreachable!("end_object called outside of an object"),
        }
        self.writer.write_all(b"}")?;
        self.end_value()?;
        Ok(())
    }

    /// Write a key for the current object.
    pub fn write_key(&mut self, key: &str) -> std::io::Result<()> {
        self.write_str(key)
    }

    write_numeric!(write_u8(value: u8));
    write_numeric!(write_u16(value: u16));
    write_numeric!(write_u32(value: u32));
    write_numeric!(write_u64(value: u64));
    write_numeric!(write_i8(value: i8));
    write_numeric!(write_i16(value: i16));
    write_numeric!(write_i32(value: i32));
    write_numeric!(write_i64(value: i64));
    write_numeric!(write_f32(value: f32));
    write_numeric!(write_f64(value: f64));

    /// Write a string value.
    pub fn write_str(&mut self, value: &str) -> std::io::Result<()> {
        self.start_value()?;
        self.writer.write_all(b"\"")?;
        // TODO escape the string properly
        write_escaped_string(&mut self.writer, value)?;
        self.writer.write_all(b"\"")?;
        self.end_value()?;
        Ok(())
    }

    /// Start new JSON value.
    fn start_value(&mut self) -> std::io::Result<()> {
        let last = self.stack.last_mut().expect("Multiple root values defined");
        match last {
            StackItem::Root => {}
            StackItem::ArrayItem { first } => {
                if *first {
                    *first = false;
                } else {
                    self.writer.write_all(b",")?;
                }
            }
            StackItem::ObjectItem { next } => match *next {
                ObjectItemNext::FirstKey => {
                    *next = ObjectItemNext::Value;
                }
                ObjectItemNext::Key => {
                    self.writer.write_all(b",")?;
                    *next = ObjectItemNext::Value;
                }
                ObjectItemNext::Value => {
                    self.writer.write_all(b":")?;
                    *next = ObjectItemNext::Key;
                }
            },
        }
        Ok(())
    }

    /// End current JSON value.
    fn end_value(&mut self) -> std::io::Result<()> {
        self.stack.pop_if(|item| *item == StackItem::Root);
        Ok(())
    }
}

/// Display a string as a JSON-escaped string.
pub struct JsonString<'a>(pub &'a str);

impl std::fmt::Display for JsonString<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Write;

        let JsonString(value) = *self;

        for c in value.chars() {
            match c {
                '"' => f.write_str("\\\"")?,
                '\\' => f.write_str("\\\\")?,
                '\n' => f.write_str("\\n")?,
                '\r' => f.write_str("\\r")?,
                '\t' => f.write_str("\\t")?,
                '\u{08}' => f.write_str("\\b")?,
                '\u{0C}' => f.write_str("\\f")?,
                c if c.is_ascii_control() => write!(f, "\\u{:04x}", c as u32)?,
                c => f.write_char(c)?,
            }
        }
        Ok(())
    }
}

fn write_escaped_string<W>(writer: &mut W, value: &str) -> std::io::Result<()>
where
    W: std::io::Write,
{
    for c in value.chars() {
        match c {
            '"' => writer.write_all(b"\\\"")?,
            '\\' => writer.write_all(b"\\\\")?,
            '\n' => writer.write_all(b"\\n")?,
            '\r' => writer.write_all(b"\\r")?,
            '\t' => writer.write_all(b"\\t")?,
            '\u{08}' => writer.write_all(b"\\b")?,
            '\u{0C}' => writer.write_all(b"\\f")?,
            c if c.is_ascii_control() => write_hexcode(writer, c)?,
            c if c.is_ascii() => writer.write_all(&[c as u8])?,
            c => write_char(writer, c)?,
        }
    }
    Ok(())
}

fn write_hexcode<W>(writer: &mut W, value: char) -> std::io::Result<()>
where
    W: std::io::Write,
{
    let value = value as u32;
    let hex = |d: u32| char::from_digit(d, 16).unwrap() as u8;
    writer.write_all(&[
        b'\\',
        b'u',
        hex((value >> 12) & 0xF),
        hex((value >> 8) & 0xF),
        hex((value >> 4) & 0xF),
        hex(value & 0xF),
    ])
}

fn write_char<W>(writer: &mut W, value: char) -> std::io::Result<()>
where
    W: std::io::Write,
{
    let mut buf = [0; size_of::<char>()];
    writer.write_all(value.encode_utf8(&mut buf).as_bytes())
}
