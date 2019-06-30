// Copyright 2017 rust-hyper-multipart-rfc7578 Developers
// Copyright 2018 rust-multipart-rfc7578 Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//

//! This crate contains an implementation of the multipart/form-data media
//! type described in [RFC 7578](https://tools.ietf.org/html/rfc7578).
//!
//! ## Usage
//!
//! ```toml
//! [dependencies]
//! multipart-rfc7578 = "0.6"
//! ```
//!
//! ```rust
//! # extern crate multipart_rfc7578;
//!
//! use multipart_rfc7578::Form;
//!
//! # fn main() {
//! let mut form = Form::default();
//!
//! form.add_text("test", "Hello World");
//! # }
//! ```
//!
mod boundary_generator;
mod form;
mod form_reader;
mod part;

#[cfg(feature = "futures")]
mod body;

#[cfg(feature = "futures")]
pub use crate::body::Body;
pub use crate::boundary_generator::{BoundaryGenerator, RandomAsciiGenerator};
pub use crate::form::Form;

pub(crate) const CRLF: &str = "\r\n";
