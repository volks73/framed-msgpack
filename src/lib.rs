//! Frame a MessagePack message (payload) from a stream with the total message length prefixed as
//! a 32-bit unsigned integer encoded in four (4) bytes.
//!
//! The implementation is based on the `tokio_io::codec::length_delimited` module in the
//! [tokio-io](https://github.com/tokio-rs/tokio-io) project but instead of decoding and encoding
//! a payload from a stream into a buffer of bytes, the full MessagePack message is decoded or
//! encoded, respectively. 
//!
//! # Getting Started
//!
//! This can be used with the tokio-proto crate.
//!
//! ```
//! use framed_msgpack;
//! use tokio_io::{AsyncRead, AsyncWrite};
//! use tokio_proto::streaming::pipeline::ServerProto;
//!
//! struct FramedMsgpackProto;
//! 
//! impl<T: AsyncRead + AsyncWrite + 'static> ServerProto<T> for FramedMsgpackProto {
//!    type Request = Value;
//!    type Response = Value;
//!    type Transport = Framed<T, framed_msgpack::Framed>;
//!    type BindTransport = Result<Self::Transport, io::Error>;
//!
//!    fn bind_transport(&self, io: T) -> Self::BindTransport {
//!        Ok(framed_msgpack::Framed::new(io))
//!    }
//! }
//! ```

extern crate bytes;
#[macro_use]
extern crate futures;
extern crate rmpv;
#[macro_use]
extern crate tokio_io;

pub use self::codec::Framed;
pub use self::codec::FramedWrite;
pub use self::codec::FramedRead;

mod codec;

