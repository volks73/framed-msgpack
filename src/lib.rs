//! Frame a MessagePack message (payload) from a transport with the total message length prefixed as
//! a 32-bit unsigned integer encoded in four (4) bytes.
//!
//! The implementation is based on the `tokio_io::codec::length_delimited` module in the
//! [tokio-io](https://github.com/tokio-rs/tokio-io) project but instead of decoding and encoding
//! a payload from a transport into a buffer of bytes, the full MessagePack message is decoded or
//! encoded, respectively. 
//!
//! # Getting Started
//!
//! This can be used with the tokio-proto crate.
//!
//! ```
//! use framed_msgpack::Codec;
//! use tokio_io::{AsyncRead, AsyncWrite};
//! use tokio_io::codec::Framed;
//! use tokio_proto::pipeline::ServerProto;
//!
//! struct FramedMsgpackProto;
//! 
//! impl<T: AsyncRead + AsyncWrite + 'static> ServerProto<T> for FramedMsgpackProto {
//!    type Request = Value;
//!    type Response = Value;
//!    type Transport = Framed<T, Codec>
//!    type BindTransport = Result<Self::Transport, io::Error>;
//!
//!    fn bind_transport(&self, io: T) -> Self::BindTransport {
//!        Ok(io.framed(Codec::new()))
//!    }
//! }
//! ```

extern crate bytes;
extern crate futures;
extern crate rmpv;
extern crate tokio_io;

pub use self::codec::Codec;

mod codec;

