// Copyright 2017 rust-hyper-multipart-rfc7578 Developers
// Copyright 2018 rust-multipart-rfc7578 Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//

use crate::form::Form;
use bytes::{BufMut, Bytes, BytesMut};
use futures::{stream::Stream, Async, Poll};
#[cfg(feature = "hyper")]
use hyper::body::Payload;
use std::io::{self, Read};

/// Multipart body that is compatible with Hyper and Actix-web.
///
pub struct Body<'a> {
    /// The amount of data to write with each chunk.
    ///
    buf_size: usize,

    /// The reader.
    ///
    reader: Box<'a + Read + Send>,
}

impl<'a> Stream for Body<'a> {
    type Item = Bytes;

    type Error = io::Error;

    /// Iterate over each form part, and write it out.
    ///
    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        let bytes = BytesMut::with_capacity(self.buf_size);
        let mut writer = bytes.writer();
        unsafe {
            let buf = writer.get_mut();
            let num = self.reader.read(&mut buf.bytes_mut())?;
            if num == 0 {
                return Ok(Async::Ready(None));
            } else {
                buf.advance_mut(num);
            }
        }
        Ok(Async::Ready(Some(writer.into_inner().freeze())))
    }
}

#[cfg(feature = "hyper")]
impl Payload for Body<'static> {
    type Data = std::io::Cursor<Bytes>;

    type Error = io::Error;

    /// Implement `Payload` so `Body` can be used with a hyper client.
    ///
    #[inline]
    fn poll_data(&mut self) -> Poll<Option<Self::Data>, Self::Error> {
        match self.poll() {
            Ok(Async::Ready(read)) => Ok(Async::Ready(read.map(bytes::IntoBuf::into_buf))),
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(e) => Err(e),
        }
    }
}

impl<'a> From<Form<'a>> for Body<'a> {
    /// Turns a `Form` into a multipart `Body`.
    ///
    #[inline]
    fn from(form: Form<'a>) -> Self {
        Self {
            buf_size: 2048,
            reader: Box::new(form.into_reader()),
        }
    }
}
