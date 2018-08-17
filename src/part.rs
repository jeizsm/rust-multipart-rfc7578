// Copyright 2017 rust-hyper-multipart-rfc7578 Developers
// Copyright 2018 rust-multipart-rfc7578 Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//

use http::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
use mime::{self, Mime};
use std::{
    fmt::Display,
    io::{Cursor, Read},
};
use CRLF;

/// One part of a body delimited by a boundary line.
///
/// [See RFC2046 5.1](https://tools.ietf.org/html/rfc2046#section-5.1).
///
pub(crate) struct Part {
    inner: Inner,

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

impl Part {
    /// Internal method to build a new Part instance. Sets the disposition type,
    /// content-type, and the disposition parameters for name, and optionally
    /// for filename.
    ///
    /// Per [4.3](https://tools.ietf.org/html/rfc7578#section-4.3), if multiple
    /// files need to be specified for one form field, they can all be specified
    /// with the same name parameter.
    ///
    pub(crate) fn new<N, F>(inner: Inner, name: N, mime: Option<Mime>, filename: Option<F>) -> Part
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

    fn headers_string(&self) -> String {
        format!(
            "{}: {}{}{}: {}{}{}",
            CONTENT_DISPOSITION.as_str(),
            self.content_disposition,
            CRLF,
            CONTENT_TYPE.as_str(),
            self.content_type,
            CRLF,
            CRLF
        )
    }

    pub(crate) fn into_reader(self) -> impl Read {
        let cursor = Cursor::new(self.headers_string());
        let inner = match self.inner {
            Inner::Text(string) => Box::new(Cursor::new(string.into_bytes())),
            Inner::Read(read, _) => read,
        };
        cursor.chain(inner).chain(Cursor::new(CRLF))
    }
}

pub(crate) enum Inner {
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
    Read(Box<'static + Read + Send>, Option<u64>),

    /// The `String` variant handles "text/plain" form data payloads.
    ///
    Text(String),
}

impl Inner {
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
