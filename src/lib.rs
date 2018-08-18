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
//! multipart-rfc7578 = "0.3.0"
//! ```
//!
//! Because the name of this library is really wordy, I recommend shortening it:
//!
//! ```rust
//! extern crate multipart_rfc7578 as multipart;
//! ```
//!
//! ```rust
//! # extern crate multipart_rfc7578;
//!
//! use multipart_rfc7578::MultipartForm;
//!
//! # fn main() {
//! let mut form = MultipartForm::default();
//!
//! form.add_text("test", "Hello World");
//! # }
//! ```
//!
#[cfg(feature = "actix-web")]
extern crate actix_web;
#[cfg(feature = "bytes")]
extern crate bytes;
#[cfg(feature = "futures")]
extern crate futures;
extern crate http;
#[cfg(feature = "hyper")]
extern crate hyper;
extern crate mime;
extern crate rand;

#[cfg(feature = "futures")]
mod body;
pub mod boundary_generator;
mod chain;
pub mod multipart;
mod part;

pub use boundary_generator::{BoundaryGenerator, RandomAsciiGenerator};
pub use multipart::MultipartForm;

pub(crate) const CRLF: &str = "\r\n";
