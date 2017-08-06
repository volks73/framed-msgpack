// Large portions of this code were obtain and modified from the tokio-io project. Specifically, the
// implementation is based on the code in the `tokio_io::codec::length_delimited` module. See
// https://github.com/tokio-rs/tokio-io.

use bytes::{Buf, BufMut, BytesMut, IntoBuf, BigEndian};
use bytes::buf::Chain;
use futures::{Async, AsyncSink, Stream, Sink, StartSend, Poll};
use rmpv::{self, Value};
use std::fmt;
use std::io::{self, Cursor};
use tokio_io::{AsyncRead, AsyncWrite, codec};

/// The number of bytes for the length prefix. The length is a 32-bit unsigned integer, which would
/// be four (4) bytes. This cannot currently be changed.
const HEAD_LENGTH: usize = 4;

pub struct Framed<T> {
    inner: FramedRead<FramedWrite<T>>,
}

impl<T: AsyncRead + AsyncWrite> Framed<T> {
    /// Creates a new `Framed`.
    pub fn new(upstream: T) -> Framed<T> {
        Framed {
            inner: FramedRead::new(FramedWrite::new(upstream)),
        }
    }

    /// Returns a reference to the underlying I/O stream wrapped by `Framed`.
    ///
    /// Note that care should be taken to not tamper with the underlying stream of data coming in
    /// as it may corrupt the stream of frames otherwise being worked with.
    pub fn get_ref(&self) -> &T {
        self.inner.get_ref().get_ref()
    }

    /// Returns a mutable reference to the underlying I/O stream wrapped by
    /// `Framed`.
    ///
    /// Note that care should be taken to not tamper with the underlying stream of data coming in
    /// as it may corrupt the stream of frames otherwise being worked with.
    pub fn get_mut(&mut self) -> &mut T {
        self.inner.get_mut().get_mut()
    }

    /// Consumes the `Framed`, returning its underlying I/O stream.
    ///
    /// Note that care should be taken to not tamper with the underlying stream of data coming in
    /// as it may corrupt the stream of frames otherwise being worked with.
    pub fn into_inner(self) -> T {
        self.inner.into_inner().into_inner()
    }
}

impl<T: AsyncRead> Stream for Framed<T> {
    type Item = Value;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, io::Error> {
        self.inner.poll()
    }
}

impl<T: AsyncWrite> Sink for Framed<T> {
    type SinkItem = Value;
    type SinkError = io::Error;

    fn start_send(&mut self, item: Self::SinkItem) -> StartSend<Self::SinkItem, io::Error> {
        self.inner.start_send(item)
    }

    fn poll_complete(&mut self) -> Poll<(), io::Error> {
        self.inner.poll_complete()
    }

    fn close(&mut self) -> Poll<(), io::Error> {
        self.inner.close()
    }
}

impl<T> fmt::Debug for Framed<T>
    where T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Framed")
            .field("inner", &self.inner)
            .finish()
    }
}


#[derive(Debug)]
pub struct FramedRead<T> {
    inner: codec::FramedRead<T, Decoder>,
}

impl<T: AsyncRead> FramedRead<T> {
    /// Creates a new `FramedRead`.
    pub fn new(upstream: T) -> FramedRead<T> {
        FramedRead {
            inner: codec::FramedRead::new(upstream, Decoder {
                state: DecodeState::Head,
            })
        }
    }

    /// Returns a reference to the underlying I/O stream wrapped by `FramedRead`.
    ///
    /// Note that care should be taken to not tamper with the underlying stream of data coming in
    /// as it may corrupt the stream of frames otherwise being worked with.
    pub fn get_ref(&self) -> &T {
        self.inner.get_ref()
    }

    /// Returns a mutable reference to the underlying I/O stream wrapped by
    /// `FramedRead`.
    ///
    /// Note that care should be taken to not tamper with the underlying stream of data coming in
    /// as it may corrupt the stream of frames otherwise being worked with.
    pub fn get_mut(&mut self) -> &mut T {
        self.inner.get_mut()
    }

    /// Consumes the `FramedRead`, returning its underlying I/O stream.
    ///
    /// Note that care should be taken to not tamper with the underlying stream of data coming in
    /// as it may corrupt the stream of frames otherwise being worked with.
    pub fn into_inner(self) -> T {
        self.inner.into_inner()
    }
}

impl<T: AsyncRead> Stream for FramedRead<T> {
    type Item = Value;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, io::Error> {
        self.inner.poll()
    }
}

impl<T: Sink> Sink for FramedRead<T> {
    type SinkItem = T::SinkItem;
    type SinkError = T::SinkError;

    fn start_send(&mut self, item: T::SinkItem) -> StartSend<T::SinkItem, T::SinkError> {
        self.inner.start_send(item)
    }

    fn poll_complete(&mut self) -> Poll<(), T::SinkError> {
        self.inner.poll_complete()
    }

    fn close(&mut self) -> Poll<(), T::SinkError> {
        self.inner.close()
    }
}

impl<T: io::Write> io::Write for FramedRead<T> {
    fn write(&mut self, src: &[u8]) -> io::Result<usize> {
        self.inner.get_mut().write(src)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.get_mut().flush()
    }
}

impl<T: AsyncWrite> AsyncWrite for FramedRead<T> {
    fn shutdown(&mut self) -> Poll<(), io::Error> {
        self.inner.get_mut().shutdown()
    }

    fn write_buf<B: Buf>(&mut self, buf: &mut B) -> Poll<usize, io::Error> {
        self.inner.get_mut().write_buf(buf)
    }
}

#[derive(Debug)]
struct Decoder {
    state: DecodeState,
}

#[derive(Debug, Clone, Copy)]
enum DecodeState {
    Head,
    Data(usize),
}

impl Decoder {
    fn decode_head(&mut self, src: &mut BytesMut) -> io::Result<Option<usize>> {
        if src.len() < HEAD_LENGTH {
            // Not enough data
            return Ok(None);
        }

        let n = {
            let mut src = Cursor::new(&mut *src);
            src.get_uint::<BigEndian>(HEAD_LENGTH)
        } as usize;

        // Ensure that the buffer has enough space to read the incoming
        // payload
        src.reserve(n);
        return Ok(Some(n));
    }

    fn decode_data(&self, n: usize, src: &mut BytesMut) -> io::Result<Option<Value>> {
        if src.len() < n {
            return Ok(None);
        }
        let v = {
            let mut src = Cursor::new(&mut *src);
            rmpv::decode::value::read_value(&mut src).map_err(|e| {
                io::Error::new(io::ErrorKind::InvalidData, format!("Decoding: {}", e))
            })
        }?;
        Ok(Some(v))
    }
}

impl codec::Decoder for Decoder {
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

pub struct FramedWrite<T, B: IntoBuf = BytesMut> {
    inner: T,
    frame: Option<Chain<Cursor<BytesMut>, B::Buf>>,
}

impl<T> FramedWrite<T> {
    /// Creates a new `FramedWrite`.
    pub fn new(upstream: T) -> FramedWrite<T> {
        FramedWrite {
            inner: upstream,
            frame: None,
        }
    }

    /// Returns a reference to the underlying I/O stream wrapped by `FramedWrite`.
    ///
    /// Note that care should be taken to not tamper with the underlying stream of data coming in
    /// as it may corrupt the stream of frames otherwise being worked with.
    pub fn get_ref(&self) -> &T {
        &self.inner
    }

    /// Returns a mutable reference to the underlying I/O stream wrapped by `FramedWrite`.
    ///
    /// Note that care should be taken to not tamper with the underlying stream of data coming in
    /// as it may corrupt the stream of frames otherwise being worked with.
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    /// Consumes the `FramedWrite`, returning its underlying I/O stream.
    ///
    /// Note that care should be taken to not tamper with the underlying stream of data coming in
    /// as it may corrupt the stream of frames otherwise being worked with.
    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T: AsyncWrite> FramedWrite<T> {
    fn do_write(&mut self) -> Poll<(), io::Error> {
        if self.frame.is_none() {
            return Ok(Async::Ready(()));
        }

        loop {
            let frame = self.frame.as_mut().unwrap();
            try_ready!(self.inner.write_buf(frame));

            if !frame.has_remaining() {
                break;
            }
        }

        self.frame = None;

        Ok(Async::Ready(()))
    }

    fn set_frame(&mut self, v: Value) -> io::Result<()> {
        let mut head = BytesMut::with_capacity(HEAD_LENGTH);
        let mut data = BytesMut::new().writer();
        rmpv::encode::write_value(&mut data, &v)?; 
        let data = data.into_inner();
        head.put_u32::<BigEndian>(data.len() as u32);
        debug_assert!(self.frame.is_none());
        self.frame = Some(head.into_buf().chain(data));
        Ok(())
    }
}

impl<T: AsyncWrite> Sink for FramedWrite<T> {
    type SinkItem = Value;
    type SinkError = io::Error;

    fn start_send(&mut self, item: Self::SinkItem) -> StartSend<Self::SinkItem, io::Error> {
        if !try!(self.do_write()).is_ready() {
            return Ok(AsyncSink::NotReady(item));
        }
        try!(self.set_frame(item));
        Ok(AsyncSink::Ready)
    }

    fn poll_complete(&mut self) -> Poll<(), io::Error> {
        try_ready!(self.do_write());
        try_nb!(self.inner.flush());
        return Ok(Async::Ready(()));
    }

    fn close(&mut self) -> Poll<(), io::Error> {
        try_ready!(self.poll_complete());
        self.inner.shutdown()
    }
}

impl<T: Stream> Stream for FramedWrite<T> {
    type Item = T::Item;
    type Error = T::Error;

    fn poll(&mut self) -> Poll<Option<T::Item>, T::Error> {
        self.inner.poll()
    }
}

impl<T: io::Read> io::Read for FramedWrite<T> {
    fn read(&mut self, dst: &mut [u8]) -> io::Result<usize> {
        self.get_mut().read(dst)
    }
}

impl<T: AsyncRead> AsyncRead for FramedWrite<T> {
    fn read_buf<B: BufMut>(&mut self, buf: &mut B) -> Poll<usize, io::Error> {
        self.get_mut().read_buf(buf)
    }

    unsafe fn prepare_uninitialized_buffer(&self, buf: &mut [u8]) -> bool {
        self.get_ref().prepare_uninitialized_buffer(buf)
    }
}

impl<T> fmt::Debug for FramedWrite<T>
    where T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("FramedWrite")
            .field("inner", &self.inner)
            .field("frame", &self.frame)
            .finish()
    }
}

