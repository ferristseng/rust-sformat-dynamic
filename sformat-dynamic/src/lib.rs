//! # sformat-dynamic
//!
//! Dynamic string formatting for Rust.
//!
//! This crate tries to mimic the [`std::fmt`](https://doc.rust-lang.org/std/fmt/#usage)
//! syntax as closely as possible. One notable differentiation thus far is that the
//! dynamic string format only supports named parameters like `{name}`, and not positional
//! parameters like `{}`.
//!
//! Always use the compile-time `format!` macro if you can. There are some situations
//! where you might need a dynamic string formatting engine, for example if you need
//! a lightweight library way for users to specify their own formats in a config file
//! or something similar. Again, using dynamic string formatting is far less secure
//! than using the compile-time `format!` macro, so always prefer that!
//!
//! **This is still a WIP, so use at your own risk!**
//!
//! ## Examples
//!
//! See `./sformat-dynamic/examples` for more examples.
//!
//! ```rust,ignore
//! // Note: This example requires the "derive" feature.
//! use sformat_dynamic::{compile, derive::Context};
//!
//! #[derive(Context)]
//! struct ContextImpl {
//!   value: usize
//! }
//!
//! let context = ContextImpl { value: 99 };
//! let format = compile("Value is {value:+010}").unwrap();
//! let formatted = format.format_str(&context).unwrap();
//!
//! assert_eq!(formatted, "Value is +000000099");
//! ```
//!
//! You can also use a `HashMap` when specifying the context, although it is significantly
//! more verbose.
//!
//! ```rust
//! use sformat_dynamic::{compile, TypedValue};
//! use std::collections::HashMap;
//!
//! let context = HashMap::from([
//!     ("name", TypedValue::Str("Ferris"))
//! ]);
//! let format = compile("Hello {name: >16}!").unwrap();
//! let formatted = format.format_str(&context).unwrap();
//!
//! assert_eq!(formatted, "Hello           Ferris!")
//! ```

#![forbid(unsafe_code)]

mod compile;
mod context;
mod format;
mod token;

#[cfg(feature = "serde")]
pub mod serde;

pub type Name<'a> = &'a str;

pub use compile::{compile, CompileError, CompiledFormat};
pub use context::{Context, DynPointer, TypedValue};
pub use format::Error as FormatError;

#[cfg(feature = "derive")]
pub mod derive {
    pub use sformat_dynamic_derive::Context;
}
