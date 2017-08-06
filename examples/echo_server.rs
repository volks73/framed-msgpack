//! A simple TCP echo server using framed-msgpack
//!
//! Requests are received and responses are just the requests sent back to the client.

extern crate framed_msgpack;
extern crate futures;
extern crate rmpv;
extern crate tokio_io;
extern crate tokio_proto;
extern crate tokio_service;

use framed_msgpack::Codec;
use futures::{future, Future, BoxFuture};
use rmpv::Value;
use std::io;
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_io::codec::Framed;
use tokio_proto::pipeline::ServerProto;
use tokio_proto::TcpServer;
use tokio_service::Service;

pub struct FramedMsgpackProto;

impl<T: AsyncRead + AsyncWrite + 'static> ServerProto<T> for FramedMsgpackProto {
    type Request = Value;
    type Response = Value;
    type Transport = Framed<T, Codec>;
    type BindTransport = Result<Self::Transport, io::Error>;

    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(Codec::new()))
    }
}

pub struct Echo;

impl Service for Echo {
    type Request = Value;
    type Response = Value;
    type Error = io::Error;
    type Future = BoxFuture<Self::Response, Self::Error>;

    fn call(&self, req: Self::Request) -> Self::Future {
        future::ok(req).boxed()
    }
}

fn main() {
    let addr = "0.0.0.0:12345".parse().unwrap();
    let server = TcpServer::new(FramedMsgpackProto, addr);
    server.serve(|| Ok(Echo));
}

