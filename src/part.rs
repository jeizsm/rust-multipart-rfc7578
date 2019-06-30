// Copyright 2017 rust-hyper-multipart-rfc7578 Developers
// Copyright 2018 rust-multipart-rfc7578 Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//
#![allow(clippy::borrow_interior_mutable_const)]
use crate::CRLF;
use http::header;
use mime::{self, Mime};
use std::{
    fmt::Display,
    io::{Cursor, Read},
};

/// One part of a body delimited by a boundary line.
///
/// [See RFC2046 5.1](https://tools.ietf.org/html/rfc2046#section-5.1).
///
pub(crate) struct Part<'a> {
    inner: Inner<'a>,

    /// Each part can include a content-type header field. If this
    /// is not specified, it defaults to "text/plain", or
    /// "application/octet-stream" for file data.
    ///
    /// [See](https://tools.ietf.org/html/rfc7578#section-4.4)
    ///
    content_type: String,

    /// Each part must contain a Content-Disposition header field.
    ///
    /// [See](https://tools.ietf.org/html/rfc7578#section-4.2).
    ///
    content_disposition: String,
}

impl<'a> Part<'a> {
    /// Internal method to build a new Part instance. Sets the disposition type,
    /// content-type, and the disposition parameters for name, and optionally
    /// for filename.
    ///
    /// Per [4.3](https://tools.ietf.org/html/rfc7578#section-4.3), if multiple
    /// files need to be specified for one form field, they can all be specified
    /// with the same name parameter.
    ///
    pub(crate) fn new<N, F>(
        inner: Inner<'a>,
        name: N,
        mime: Option<Mime>,
        filename: Option<F>,
    ) -> Part
    where
        N: Display,
        F: Display,
    {
        // `name` disposition parameter is required. It should correspond to the
        // name of a form field.
        //
        // [See 4.2](https://tools.ietf.org/html/rfc7578#section-4.2)
        //
        let mut disposition_params = vec![format!("name=\"{}\"", name)];

        // `filename` can be supplied for files, but is totally optional.
        //
        // [See 4.2](https://tools.ietf.org/html/rfc7578#section-4.2)
        //
        if let Some(filename) = filename {
            disposition_params.push(format!("filename=\"{}\"", filename));
        }

        let content_type = format!("{}", mime.unwrap_or_else(|| inner.default_content_type()));
        Part {
            inner,
            content_type,
            content_disposition: format!("form-data; {}", disposition_params.join("; ")),
        }
    }

    #[inline]
    fn headers_string(&self) -> String {
        #[cfg(feature = "part-content-length")]
        let content_length = match self.inner.len() {
            Some(len) => format!("{}{}: {}", CRLF, header::CONTENT_LENGTH.as_str(), len),
            None => String::new(),
        };
        #[cfg(not(feature = "part-content-length"))]
        let content_length = "";
        format!(
            "{}: {}{}{}: {}{}{}{}",
            header::CONTENT_DISPOSITION.as_str(),
            self.content_disposition,
            CRLF,
            header::CONTENT_TYPE.as_str(),
            self.content_type,
            content_length,
            CRLF,
            CRLF
        )
    }

    pub(crate) fn into_reader(self) -> impl Read + 'a {
        let cursor = Cursor::new(self.headers_string());
        let inner = match self.inner {
            Inner::Text(string) => Box::new(Cursor::new(string.into_bytes())),
            Inner::Read(read, _) => read,
        };
        cursor.chain(inner).chain(Cursor::new(CRLF))
    }

    #[inline]
    fn content_disposition_len(&self) -> u64 {
        (header::CONTENT_DISPOSITION.as_str().len() + 2 + self.content_disposition.len() + 2) as u64
    }

    #[inline]
    fn content_type_len(&self) -> u64 {
        (header::CONTENT_TYPE.as_str().len() + 2 + self.content_type.len() + 2) as u64
    }

    #[inline]
    fn content_length_len(&self) -> u64 {
        #[cfg(feature = "part-content-length")]
        return (header::CONTENT_LENGTH.as_str().len()
            + 2
            + self.inner.len().unwrap().to_string().len()
            + 2) as u64;
        #[cfg(not(feature = "part-content-length"))]
        0
    }

    #[inline]
    pub(crate) fn content_length(&self) -> Option<u64> {
        self.inner.len().map(|len| {
            len + self.content_disposition_len()
                + self.content_length_len()
                + self.content_type_len()
                + 2
        })
    }
}

pub(crate) enum Inner<'a> {
    /// The `Read` variant captures multiple cases.
    ///
    ///   * The first is it supports uploading a file, which is explicitly
    ///     described in RFC 7578.
    ///
    ///   * The second (which is not described by RFC 7578), is it can handle
    ///     arbitrary input streams (for example, a server response).
    ///     Any arbitrary input stream is automatically considered a file,
    ///     and assigned the corresponding content type if not explicitly
    ///     specified.
    ///
    Read(Box<'a + Read + Send>, Option<u64>),

    /// The `String` variant handles "text/plain" form data payloads.
    ///
    Text(String),
}

impl<'a> Inner<'a> {
    /// Returns the default content-type header value as described in section 4.4.
    ///
    /// [See](https://tools.ietf.org/html/rfc7578#section-4.4)
    ///
    #[inline]
    fn default_content_type(&self) -> Mime {
        match *self {
            Inner::Read(_, _) => mime::APPLICATION_OCTET_STREAM,
            Inner::Text(_) => mime::TEXT_PLAIN,
        }
    }

    /// Returns the length of the inner type.
    ///
    #[inline]
    fn len(&self) -> Option<u64> {
        match *self {
            Inner::Read(_, len) => len,
            Inner::Text(ref s) => Some(s.len() as u64),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Inner, Part};
    use std::io::{Cursor, Read};
    #[test]
    fn test_inner_text() {
        let name = "hello";
        let inner_content = "world";
        let inner = Inner::Text(inner_content.to_string());
        #[cfg(feature = "part-content-length")]
        let test_string = "content-disposition: form-data; name=\"hello\"\r
content-type: text/plain\r
content-length: 5\r
\r
world\r
";
        #[cfg(not(feature = "part-content-length"))]
        let test_string = "content-disposition: form-data; name=\"hello\"\r
content-type: text/plain\r
\r
world\r
";
        let test_string = test_string.to_string();
        let part = Part::new::<_, &str>(inner, name, None, None);
        let mut part_string = String::new();
        part.into_reader().read_to_string(&mut part_string).unwrap();
        assert_eq!(test_string, part_string);
    }

    #[test]
    fn test_inner_read() {
        let name = "hello";
        let inner_content = "world";
        let inner = Inner::Read(
            Box::new(Cursor::new(inner_content.as_bytes())),
            Some(inner_content.len() as u64),
        );
        #[cfg(feature = "part-content-length")]
        let test_string = "content-disposition: form-data; name=\"hello\"\r
content-type: application/octet-stream\r
content-length: 5\r
\r
world\r
";
        #[cfg(not(feature = "part-content-length"))]
        let test_string = "content-disposition: form-data; name=\"hello\"\r
content-type: application/octet-stream\r
\r
world\r
";
        let test_string = test_string.to_string();
        let part = Part::new::<_, &str>(inner, name, None, None);
        let mut part_string = String::new();
        part.into_reader().read_to_string(&mut part_string).unwrap();
        assert_eq!(test_string, part_string);
    }
}
