use std::io::{Bytes, Read};

#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct ByteLines<R>(Bytes<R>);
impl<R: Read> Iterator for ByteLines<R> {
    type Item = Vec<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut line = Vec::new();
        for byte in &mut self.0 {
            let byte = byte.expect("Should be able to read bytes");
            if byte == b'\n' {
                return Some(line);
            }
            line.push(byte);
        }
        if line.is_empty() {
            None
        } else {
            Some(line)
        }
    }
}

#[allow(clippy::module_name_repetitions)]
pub trait ReadByteLines<R> {
    fn byte_lines(self) -> ByteLines<R>;
}

impl<R: Read> ReadByteLines<R> for R {
    fn byte_lines(self) -> ByteLines<R> {
        ByteLines(self.bytes())
    }
}
