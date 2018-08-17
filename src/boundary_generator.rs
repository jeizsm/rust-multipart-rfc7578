// Copyright 2017 rust-hyper-multipart-rfc7578 Developers
// Copyright 2018 rust-multipart-rfc7578 Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//

use rand::{distributions::Alphanumeric, rngs::SmallRng, FromEntropy, Rng};
use std::iter::FromIterator;

/// A `BoundaryGenerator` is a policy to generate a random string to use
/// as a part boundary.
///
/// The default generator will build a random string of 6 ascii characters.
/// If you need more complexity, you can implement this, and use it with
/// [`Form::new`](/multipart_rfc7578/multipart/struct.MultipartForm.html#method.new).
///
/// # Examples
///
/// ```
/// use multipart_rfc7578::BoundaryGenerator;
///
/// struct TestGenerator;
///
/// impl BoundaryGenerator for TestGenerator {
///     fn generate_boundary() -> String {
///         "test".to_string()
///     }
/// }
/// ```
pub trait BoundaryGenerator {
    /// Generates a String to use as a boundary.
    ///
    fn generate_boundary() -> String;
}

pub struct RandomAsciiGenerator;

impl BoundaryGenerator for RandomAsciiGenerator {
    /// Creates a boundary of 6 ascii characters.
    ///
    fn generate_boundary() -> String {
        let mut rng = SmallRng::from_entropy();
        let ascii = rng.sample_iter(&Alphanumeric);

        String::from_iter(ascii.take(6))
    }
}
