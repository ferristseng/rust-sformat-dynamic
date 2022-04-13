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
//! ### Feature Parity
//!
//! Consult the [`str::fmt`](https://doc.rust-lang.org/std/fmt/) documentation for
//! what these features actually mean.
//!
//! | Feature                             | Implemented | Future Plan to Implement |
//! | ----------------------------------- | ----------- | ------------------------ |
//! | Named Argument       `{name}`       | ✅          | N/A                      |
//! | Positional Argument  `{}`           | ❌          | ❌                       |
//! | Fill / Alignment     `< | ^ | >`    | ✅          | N/A                      |
//! | Sign Flag            `+`            | ✅          | N/A                      |
//! | Alternate Form Flag  `#`            | ❌          | 🤔                       |
//! | Zero Flag            `0`            | ✅          | N/A                      |
//! | Precision - Fixed    `.N`           | ✅          | N/A                      |
//! | Precision - Arg      `.N$`          | ❌          | ❌                       |
//! | Precision - Astrix   `.*`           | ❌          | ❌                       |
//!
//! ### Derive Types
//! 
//! A table of types that can be derived using `sformat-dynamic-derive`.
//!
//! | Type                                | Implemented | Future Plan to Implement |
//! | ----------------------------------- | ----------- | ------------------------ |
//! | &T : Debug                          | ❌          | ✅                       |
//! | &T : Display                        | ❌          | ✅                       |
//! | &str                                | ✅          | ✅                       |
//! | isize                               | ✅          | N/A                      |
//! | i64                                 | ✅          | N/A                      |
//! | i32                                 | ✅          | N/A                      |
//! | i16                                 | ✅          | N/A                      |
//! | i8                                  | ✅          | N/A                      |
//! | usize                               | ✅          | N/A                      |
//! | u64                                 | ✅          | N/A                      |
//! | u32                                 | ✅          | N/A                      |
//! | u16                                 | ✅          | N/A                      |
//! | u8                                  | ✅          | N/A                      |
//! | f64                                 | ✅          | N/A                      |
//! | f32                                 | ✅          | N/A                      |
//! | bool                                | ✅          | N/A                      |

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
