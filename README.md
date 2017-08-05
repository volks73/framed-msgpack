# Framed-Msgpack: A MessagePack-based codec with a prefix payload length for tokio-rs #

## What is Framed-Msgpack

The Framed-Msgpack project is a [Rust](http://www.rust-lang.org) crate, a.k.a. library, that provides a codec for use with the [tokio-rs](https://tokio.rs/) project. [MessagePack](http://msgpack.org/index.html)-based messages, or payloads, are sent and received with the total message length prefixed as a 32-bit unsigned integer encoded in four (4) [Big-Endian](https://en.wikipedia.org/wiki/Endianness) bytes. The implementation is largely based on the [length delimited](https://docs.rs/tokio-io/0.1.2/tokio_io/codec/length_delimited/index.html) codec provided in the [tokio-io](https://docs.rs/tokio-io/0.1.2/tokio_io/) module but instead of decoding and encoding a payload from a stream and sink into a byte buffer, a payload is decoded and encoded as MessagePack messages _without_ the prefix length bytes, respectively.

## Usage ##

First, add this to your `Cargo.toml`:

```toml
[dependencies]
framed-msgpack = "*"
```

Next, add this to your crate:

```rust
extern crate framed-msgpack;
```

## License ##

See the LICENSE file for more information about licensing and copyright.

