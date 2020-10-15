<!-- cargo-sync-readme start -->

# Overview
- [📦 crates.io](https://crates.io/crates/serde-scale-wrap)
- [📖 Documentation](https://docs.rs/serde-scale-wrap)
- [⚖ zlib license](https://opensource.org/licenses/Zlib)

Wrapper for types implementing [`Serialize`/`Deserialize`](https://docs.rs/serde) to implement
[`Encode`/`Decode`](https://docs.rs/parity-scale-codec) automatically.

⚠ The `Error` type exposed by this crate is meant to disappear if/when `parity-scale-codec`'s
`Error` implements `Display` unconditionally.

# Example
```rust
extern crate alloc;

use alloc::string::String;
use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};
use serde_scale_wrap::Wrap;

#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct Foo {
    x: i32,
    s: String,
}

let original = Foo { x: 3, s: "foo".into() };
let serialized = Wrap(&original).encode();
let Wrap(deserialized) = Wrap::<Foo>::decode(&mut &*serialized).unwrap();
assert_eq!(original, deserialized);
```

# Conformance
⚠ `Option<bool>` is serialized as a single byte according to the SCALE encoding, which differs
from the result of `Encode::encode` -- `Encode` expects `OptionBool` to be used instead.

# Features
`no_std` is supported by disabling default features.

- `std`: Support for `std`. It is enabled by default.

🔖 Features enabled in build dependencies and proc-macros are also enabled for normal
dependencies, which may cause `serde` to have its `std` feature on when it is not desired.
Nightly cargo prevents this from happening with
[`-Z features=host_dep`](https://github.com/rust-lang/cargo/issues/7915#issuecomment-683294870)
or the following in `.cargo/config`:

```toml
[unstable]
features = ["host_dep"]
```

For example, this issue arises when depending on `parity-scale-codec-derive`.

# Contribute
All contributions shall be licensed under the [zlib license](https://opensource.org/licenses/Zlib).

# Related projects
- [parity-scale-codec](https://crates.io/crates/parity-scale-codec): Reference Rust implementation
- [serde-scale](https://crates.io/crates/serde-scale): SCALE encoding with `serde`

<!-- cargo-sync-readme end -->
