// Large portions of this code were obtain and modified from the tokio-io project. Specifically, the
// implementation is based on the code in the `tokio_io::codec::length_delimited` module. See
// https://github.com/tokio-rs/tokio-io.

use bytes::{Buf, BufMut, BytesMut, BigEndian};
use rmpv::{self, Value};
use std::io::{self, Cursor};
use tokio_io::codec;

/// The number of bytes for the length prefix. The length is a 32-bit unsigned integer, which would
/// be four (4) bytes. This cannot currently be changed.
// Maybe some day this could be configurable?
const HEAD_LENGTH: usize = 4;

#[derive(Debug, Clone, Copy)]
pub struct Codec {
    state: DecodeState,
}

#[derive(Debug, Clone, Copy)]
enum DecodeState {
    Head,
    Data(usize),
}

impl Codec {
    pub fn new() -> Self {
        Codec {
            state: DecodeState::Head,
        }
    }

    fn decode_head(&mut self, src: &mut BytesMut) -> io::Result<Option<usize>> {
        if src.len() < HEAD_LENGTH {
            return Ok(None);
        }
        let n = {
            let mut src = Cursor::new(&mut *src);
            src.get_uint::<BigEndian>(HEAD_LENGTH)
        } as usize;
        // Ensure that the buffer has enough space to read the incoming payload
        src.reserve(n);
        return Ok(Some(n));
    }

    fn decode_data(&self, n: usize, src: &mut BytesMut) -> io::Result<Option<Value>> {
        if src.len() < n {
            return Ok(None);
        }
        // Remove the prefixed length bytes 
        src.split_to(HEAD_LENGTH);
        let v = {
            let mut src = Cursor::new(&mut *src);
            rmpv::decode::value::read_value(&mut src).map_err(|e| {
                io::Error::new(io::ErrorKind::InvalidData, format!("Decoding: {}", e))
            })
        }?;
        // Remove the decoded data bytes
        src.split_to(n);
        Ok(Some(v))
    }
}

impl codec::Decoder for Codec {
    type Item = Value;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> io::Result<Option<Self::Item>> {
        let n = match self.state {
            DecodeState::Head => {
                match try!(self.decode_head(src)) {
                    Some(n) => {
                        self.state = DecodeState::Data(n);
                        n
                    }
                    None => return Ok(None),
                }
            }
            DecodeState::Data(n) => n,
        };

        match try!(self.decode_data(n, src)) {
            Some(data) => {
                self.state = DecodeState::Head;
                src.reserve(HEAD_LENGTH);
                Ok(Some(data))
            }
            None => Ok(None),
        }
    }
}

impl codec::Encoder for Codec {
    type Item = Value;
    type Error = io::Error;

    fn encode(&mut self, msg: Self::Item, buf: &mut BytesMut) -> io::Result<()> {
        // I wonder if there is some way to the `buf` for the value directly but skip the first
        // four bytes in the buffer and then write the bytes for the length back into the first
        // four skipped bytes? Probably not since the buffer would have to be changed to a writer,
        // which would consume it.
        let mut data: Vec<u8> = Vec::new();
        rmpv::encode::write_value(&mut data, &msg)?; 
        buf.put_u32::<BigEndian>(data.len() as u32);
        buf.extend(data);
        Ok(())
    }
}

impl Default for Codec {
    fn default() -> Self {
        Codec {
            state: DecodeState::Head,
        }
    }
}

