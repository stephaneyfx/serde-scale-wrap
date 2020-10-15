// Copyright (C) 2020 Stephane Raux. Distributed under the zlib license.

//! # Overview
//! - [📦 crates.io](https://crates.io/crates/serde-scale-wrap)
//! - [📖 Documentation](https://docs.rs/serde-scale-wrap)
//! - [⚖ zlib license](https://opensource.org/licenses/Zlib)
//!
//! Wrapper for types implementing [`Serialize`/`Deserialize`](https://docs.rs/serde) to implement
//! [`Encode`/`Decode`](https://docs.rs/parity-scale-codec) automatically.
//!
//! ⚠ The `Error` type exposed by this crate is meant to disappear if/when `parity-scale-codec`'s
//! `Error` implements `Display` unconditionally.
//!
//! # Example
//! ```rust
//! extern crate alloc;
//!
//! use alloc::string::String;
//! use parity_scale_codec::{Decode, Encode};
//! use serde::{Deserialize, Serialize};
//! use serde_scale_wrap::Wrap;
//!
//! #[derive(Debug, Deserialize, PartialEq, Serialize)]
//! struct Foo {
//!     x: i32,
//!     s: String,
//! }
//!
//! let original = Foo { x: 3, s: "foo".into() };
//! let serialized = Wrap(&original).encode();
//! let Wrap(deserialized) = Wrap::<Foo>::decode(&mut &*serialized).unwrap();
//! assert_eq!(original, deserialized);
//! ```
//!
//! # Conformance
//! ⚠ `Option<bool>` is serialized as a single byte according to the SCALE encoding, which differs
//! from the result of `Encode::encode` -- `Encode` expects `OptionBool` to be used instead.
//!
//! # Features
//! `no_std` is supported by disabling default features.
//!
//! - `std`: Support for `std`. It is enabled by default.
//!
//! 🔖 Features enabled in build dependencies and proc-macros are also enabled for normal
//! dependencies, which may cause `serde` to have its `std` feature on when it is not desired.
//! Nightly cargo prevents this from happening with
//! [`-Z features=host_dep`](https://github.com/rust-lang/cargo/issues/7915#issuecomment-683294870)
//! or the following in `.cargo/config`:
//!
//! ```toml
//! [unstable]
//! features = ["host_dep"]
//! ```
//!
//! For example, this issue arises when depending on `parity-scale-codec-derive`.
//!
//! # Contribute
//! All contributions shall be licensed under the [zlib license](https://opensource.org/licenses/Zlib).
//!
//! # Related projects
//! - [parity-scale-codec](https://crates.io/crates/parity-scale-codec): Reference Rust implementation
//! - [serde-scale](https://crates.io/crates/serde-scale): SCALE encoding with `serde`

#![deny(warnings)]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::vec::Vec;
use core::{
    convert::Infallible,
    fmt::{self, Display},
};
use parity_scale_codec::{Decode, Encode, EncodeLike, Input, Output};
use serde::{Deserialize, Serialize};
use serde_scale::{Bytes, Read, Write};

/// Wrapper for types serializable with `serde` to support serialization with `Encode`/`Decode`
///
/// This can help to pass instances of types implementing `Serialize`/`Deserialize` to `substrate`
/// functions expecting types implementing `Encode`/`Decode`.
///
/// ⚠ The `Encode` implementation panics if the serializer returns an error (e.g. when attempting
/// to serialize a floating point number) because `Encode` methods do not return `Result`.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Wrap<T>(pub T);

impl<T: Serialize> Encode for Wrap<T> {
    /// # Panics
    /// Panics if the serializer returns an error (e.g. when attempting to serialize a floating
    /// point number).
    fn encode_to<O: Output>(&self, dst: &mut O) {
        let mut serializer = serde_scale::Serializer::new(OutputToWrite(dst));
        self.0.serialize(&mut serializer).unwrap();
    }
}

impl<T: Serialize> EncodeLike for Wrap<T> {}

impl<'de, T: Deserialize<'de>> Decode for Wrap<T> {
    fn decode<I: Input>(input: &mut I) -> Result<Self, parity_scale_codec::Error> {
        let mut deserializer = serde_scale::Deserializer::new(InputToRead::new(input));
        match T::deserialize(&mut deserializer) {
            Ok(x) => Ok(Wrap(x)),
            Err(serde_scale::Error::Io(Error(s))) => Err(s.into()),
            Err(_) => Err("Deserialization failed".into()),
        }
    }
}

struct OutputToWrite<'a, O: ?Sized>(&'a mut O);

impl<O: Output + ?Sized> Write for OutputToWrite<'_, O> {
    type Error = Infallible;

    fn write(&mut self, bytes: &[u8]) -> Result<(), Infallible> {
        self.0.write(bytes);
        Ok(())
    }
}

struct InputToRead<'a, I: ?Sized> {
    input: &'a mut I,
    buffer: Vec<u8>,
}

impl<'a, I: Input + ?Sized> InputToRead<'a, I> {
    fn new(input: &'a mut I) -> Self {
        InputToRead {
            input,
            buffer: Vec::new(),
        }
    }
}

impl<'a, 'de, I: Input + ?Sized> Read<'de> for InputToRead<'a, I> {
    type Error = Error;

    fn read_map<R, F>(&mut self, n: usize, f: F) -> Result<R, Self::Error>
    where
        F: FnOnce(Bytes<'de, '_>) -> R,
    {
        self.buffer.resize(n, 0);
        self.input.read(&mut self.buffer).map_err(|e| Error(e.what()))?;
        Ok(f(Bytes::Temporary(&self.buffer)))
    }
}

/// Unstable error type meant to disappear if/when `parity-scale-codec`'s `Error` implements
/// `Display` unconditionally.
#[derive(Debug)]
pub struct Error(pub &'static str);

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    use alloc::string::String;
    use crate::Wrap;
    use parity_scale_codec::{Decode, Encode};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, PartialEq, Serialize)]
    struct Foo {
        x: i32,
        s: String,
    }

    #[test]
    fn foo_roundtrips() {
        let original = Foo { x: 3, s: "foo".into() };
        let serialized = Wrap(&original).encode();
        let Wrap(deserialized) = Wrap::<Foo>::decode(&mut &*serialized).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn foo_is_correctly_serialized() {
        let original = Foo { x: 3, s: "foo".into() };
        let wrapped_serialized = Wrap(&original).encode();
        let serialized = serde_scale::to_vec(&original).unwrap();
        assert_eq!(wrapped_serialized, serialized);
    }
}
