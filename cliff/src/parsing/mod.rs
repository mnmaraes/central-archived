use std::{pin::Pin, task::Poll};

use bytes::BytesMut;

use failure::{Error, ResultExt};

use futures::stream::Stream;

use rmpv::{self, decode::value::read_value, encode::write_value};

enum DataSize {
    S8,
    S16,
    S32,
}

fn get_be_size(buffer: &[u8], size_type: DataSize) -> usize {
    use DataSize::*;
    match size_type {
        S8 => buffer[1] as usize,
        S16 => u16::from_be_bytes([buffer[1], buffer[2]]) as usize,
        S32 => u32::from_be_bytes([buffer[1], buffer[2], buffer[3], buffer[4]]) as usize,
    }
}

fn size_compound(buffer: &[u8], count: usize, skip: usize) -> Option<usize> {
    let mut sized_elements = 0;
    let mut size = skip;

    while sized_elements < count {
        if buffer.len() <= size {
            return None;
        }

        match get_value_size(&buffer[size..]) {
            Some(s) => size += s,
            None => return None,
        }

        sized_elements += 1;
    }

    Some(size)
}

fn get_value_size(buffer: &[u8]) -> Option<usize> {
    use DataSize::*;
    assert_ne!(
        buffer.len(),
        0,
        "Invariant Violation: Can't size message without bytes"
    );

    // Match simple sized values
    match buffer[0] {
        // *Number*
        // fixint | (nil or boolean) | negative fixint
        0..=0x7f | 0xc0..=0xc3 | 0xe0..=0xff => Some(1),
        // 8-bit: uint | int
        0xcc | 0xd0 => Some(2),
        // 16-bit: uint | int
        0xcd | 0xd1 => Some(3),
        // 32-bit: float | uint | int
        0xca | 0xce | 0xd2 => Some(5),
        // 64-bit: float | uint | int
        0xcb | 0xcf | 0xd3 => Some(9),
        //
        // *String or Binary*
        // fixstring
        0xa0..=0xbf => Some(1 + (buffer[0] & 0x1f) as usize), // 101XXXXX & 00011111
        // 8-bit size: str | bin
        0xd9 | 0xc4 if buffer.len() < 2 => None,
        0xd9 | 0xc4 => Some(2 + get_be_size(&buffer, S8)),
        // 16-bit size: str | bin
        0xda | 0xc5 if buffer.len() < 3 => None,
        0xda | 0xc5 => Some(3 + get_be_size(&buffer, S16)),
        // 32-but size: str | bin
        0xdb | 0xc6 if buffer.len() < 5 => None,
        0xdb | 0xc6 => Some(5 + get_be_size(&buffer, S32)),
        //
        // *Ext*
        // fixext(1)
        0xd4 => Some(3),
        // fixext(2)
        0xd5 => Some(4),
        // fixext(4)
        0xd6 => Some(6),
        // fixext(8)
        0xd7 => Some(10),
        // fixext(16)
        0xd8 => Some(18),
        // ext(8)
        0xc7 if buffer.len() < 2 => None,
        0xc7 => Some(3 + get_be_size(&buffer, S8)),
        // ext(16)
        0xc8 if buffer.len() < 3 => None,
        0xc8 => Some(4 + get_be_size(&buffer, S16)),
        // ext(32)
        0xc9 if buffer.len() < 5 => None,
        0xc9 => Some(6 + get_be_size(&buffer, S32)),
        //
        // *Array*
        // fixarray
        0x90..=0x9f => size_compound(&buffer, (buffer[0] & 0x0f) as usize, 1),
        // array(16)
        0xdc if buffer.len() < 3 => None,
        0xdc => size_compound(&buffer, get_be_size(&buffer, S16), 3),
        // array(32)
        0xdd if buffer.len() < 5 => None,
        0xdd => size_compound(&buffer, get_be_size(&buffer, S32), 5),
        //
        // *Map*
        // fixmap
        0x80..=0x8f => size_compound(&buffer, (buffer[0] & 0x0f) as usize * 2, 1),
        // array(16)
        0xde if buffer.len() < 3 => None,
        0xde => size_compound(&buffer, get_be_size(&buffer, S16) * 2, 3),
        // array(32)
        0xdf if buffer.len() < 5 => None,
        0xdf => size_compound(&buffer, get_be_size(&buffer, S32) * 2, 5),
    }
}

pub struct MsgPackParser<R: tokio::io::AsyncRead + std::marker::Unpin> {
    _reader: R,

    unparsed_buffer: Vec<u8>,
}

impl<R: tokio::io::AsyncRead + std::marker::Unpin> MsgPackParser<R> {
    pub fn new(reader: R) -> Self {
        Self {
            _reader: reader,

            unparsed_buffer: vec![],
        }
    }

    fn parse_next(&self, buffer: &[u8]) -> Result<(Option<rmpv::Value>, Vec<u8>), Error> {
        if buffer.is_empty() {
            return Ok((None, vec![]));
        }

        let size = match get_value_size(buffer) {
            Some(size) if buffer.len() >= size => size,
            _ => return Ok((None, Vec::from(buffer))),
        };

        let (mut value_buffer, unparsed) = (&buffer[..size], &buffer[size..]);
        let value = read_value(&mut value_buffer)?;

        Ok((Some(value), Vec::from(unparsed)))
    }

    fn reader(self: &mut Self) -> Pin<&mut R> {
        Pin::new(&mut self._reader)
    }

    fn ingest(self: &mut Self, buffer: &[u8]) {
        self.unparsed_buffer.extend_from_slice(buffer);
    }

    fn read_next(self: &mut Self) -> Option<rmpv::Value> {
        // 2. Try to parse the next message from the resulting buffer
        let (next, unparsed) = self
            .parse_next(&self.unparsed_buffer[..])
            .context("Failure Parsing on byte ingest")
            .unwrap();
        // 3. Move the resulting unparsed section (even if empty) to `self.unparsed_buffer`
        self.unparsed_buffer = unparsed;

        next
    }
}

impl<R: tokio::io::AsyncRead + std::marker::Unpin> Stream for MsgPackParser<R> {
    type Item = rmpv::Value;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut futures::task::Context,
    ) -> Poll<Option<rmpv::Value>> {
        // 1. If there are unparsed values available, parse them
        if let Some(value) = self.read_next() {
            return Poll::Ready(Some(value));
        }

        // 2. Read data from `reader`
        let mut read_buffer = BytesMut::with_capacity(256);
        // 3. While the reader's polling isn't pending:
        while let Poll::Ready(result) = self.reader().poll_read_buf(cx, &mut read_buffer) {
            // TODO: Handle possible `result` error here
            let read_bytes = result.unwrap();
            //  a. If zero bytes read, the connection has been closed (close stream)
            if read_bytes == 0 {
                return Poll::Ready(None);
            }

            //  b. Otherwise, ingest available bytes
            self.ingest(&read_buffer[..read_bytes]);
            match self.read_next() {
                //  c. If Value ready, return it as next value
                Some(value) => return Poll::Ready(Some(value)),
                //  d. Otherwise, reset read_buffer and continue
                None => {
                    read_buffer = BytesMut::with_capacity(256);
                }
            };
        }

        // 4. If we can't read from `reader` anymore, return pending value
        Poll::Pending
    }
}

pub fn encode_value(value: &rmpv::Value) -> Vec<u8> {
    let mut encoded = Vec::new();

    write_value(&mut encoded, value).ok();

    encoded
}
