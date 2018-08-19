// Copyright 2018 rust-multipart-rfc7578 Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//

use std::io::{self, Cursor, Read};

enum State {
    Boundary,
    Part,
    FinalBoundary,
}

pub(crate) struct FormReader<T: Read, R: Iterator<Item = T>> {
    boundary: Cursor<String>,
    current: Option<T>,
    readers: R,
    final_boundary: Cursor<String>,
    state: State,
}

impl<T: Read, R: Iterator<Item = T>> FormReader<T, R> {
    pub(crate) fn new(
        boundary: Cursor<String>,
        mut readers: R,
        final_boundary: Cursor<String>,
    ) -> Self {
        Self {
            boundary,
            current: readers.next(),
            readers,
            final_boundary,
            state: State::Boundary,
        }
    }
}

impl<T: Read, R: Iterator<Item = T>> Read for FormReader<T, R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match &self.state {
            State::Boundary => match self.boundary.read(buf)? {
                0 => {
                    self.state = State::Part;
                    self.current.as_mut().unwrap().read(buf)
                }
                n => Ok(n),
            },
            State::Part => match self.current.as_mut().unwrap().read(buf)? {
                0 => {
                    self.current = self.readers.next();
                    match self.current {
                        Some(_) => {
                            self.boundary.set_position(0);
                            self.state = State::Boundary;
                            self.boundary.read(buf)
                        }
                        None => {
                            self.state = State::FinalBoundary;
                            self.final_boundary.read(buf)
                        }
                    }
                }
                n => Ok(n),
            },
            State::FinalBoundary => self.final_boundary.read(buf),
        }
    }
}
