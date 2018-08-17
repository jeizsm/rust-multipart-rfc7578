// Copyright 2018 rust-multipart-rfc7578 Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//

use std::io::{self, Read};

pub(crate) struct ReadersChain<T: Read, R: Iterator<Item = T>> {
    current: Option<T>,
    readers: R,
}

impl<T: Read, R: Iterator<Item = T>> ReadersChain<T, R> {
    pub(crate) fn new(mut readers: R) -> Self {
        ReadersChain {
            current: readers.next(),
            readers: readers,
        }
    }
}

impl<T: Read, R: Iterator<Item = T>> Read for ReadersChain<T, R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.current.is_none() {
            return Ok(0);
        }
        match self.current.as_mut().unwrap().read(buf)? {
            0 => {
                self.current = self.readers.next();
                self.read(buf)
            }
            n => Ok(n),
        }
    }
}
