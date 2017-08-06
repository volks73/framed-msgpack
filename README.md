# framed-msgpack #

## What is framed-msgpack?

The framed-msgpack project is a [Rust](http://www.rust-lang.org) crate, a.k.a. library, that provides a codec for use with the [tokio-rs](https://tokio.rs/) project. [MessagePack](http://msgpack.org/index.html)-based messages, or payloads, are sent and received with the total message length prefixed as a 32-bit unsigned integer encoded in four (4) [Big-Endian](https://en.wikipedia.org/wiki/Endianness) bytes. The implementation is largely based on the [length delimited](https://docs.rs/tokio-io/0.1.2/tokio_io/codec/length_delimited/index.html) codec provided in the [tokio-io](https://docs.rs/tokio-io/0.1.2/tokio_io/) module but instead of decoding and encoding a payload from a stream and sink into a byte buffer, a payload is decoded and encoded as MessagePack messages _without_ the prefix length bytes, respectively.

## Usage ##

First, add this to your `Cargo.toml`:

```toml
[dependencies]
framed-msgpack = { git = "https://github.com/volks73/framed-msgpack.git" }
```

Next, add this to your crate:

```rust
extern crate framed_msgpack;
```

## Getting Started ##

Clone this repository, then run the following command:

```
$ cargo run --example echo_server
```

This will run a server at `localhost:12345`. From another terminal, connect to the server and send framed-msgpack messages. The binary messages will be returned unchanged. A combination of the [netcat](https://en.wikipedia.org/wiki/Netcat), `nc`, application on UNIX-like systems, or [ncat](https://nmap.org/ncat/) for Windows, and the [panser](https://github.com/volks73/panser) application can be used to create framed-msgpack messages to send to the example echo server. For example, 

```
$ echo '[0,1,2]' | panser --from json --to json --sized-output | nc localhost 12345 | panser --from msgpack --to json --sized-input --delimited-output 0Ah
[0,1,2]
$
```

## License ##

See the LICENSE file for more information about licensing and copyright.

