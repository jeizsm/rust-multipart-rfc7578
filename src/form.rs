// Copyright 2017 rust-hyper-multipart-rfc7578 Developers
// Copyright 2018 rust-multipart-rfc7578 Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//

use boundary_generator::{BoundaryGenerator, RandomAsciiGenerator};
use form_reader::FormReader;
use mime::Mime;
use part::{Inner, Part};
use std::borrow::Borrow;
use std::{
    fmt::Display,
    fs::File,
    io::{self, Cursor, Read},
    path::Path,
    str::FromStr,
};
use CRLF;

#[cfg(feature = "actix-web")]
use actix_web::{
    self,
    client::{ClientRequest, ClientRequestBuilder},
};
#[cfg(any(feature = "hyper", feature = "actix-web"))]
use body::Body;
#[cfg(any(feature = "hyper", feature = "actix-web"))]
#[allow(unused_imports)]
use http::{
    self,
    header::{CONTENT_LENGTH, CONTENT_TYPE},
    request::{Builder, Request},
};
#[cfg(feature = "hyper")]
use hyper;

// use error::Error;

/// Implements the multipart/form-data media type as described by
/// RFC 7578.
///
/// [See](https://tools.ietf.org/html/rfc7578#section-1).
///
pub struct Form {
    parts: Vec<Part>,

    /// The auto-generated boundary as described by 4.1.
    ///
    /// [See](https://tools.ietf.org/html/rfc7578#section-4.1).
    ///
    boundary: String,
}

impl Default for Form {
    /// Creates a new form with the default boundary generator.
    ///
    #[inline]
    fn default() -> Form {
        Form::new::<RandomAsciiGenerator>()
    }
}

impl Form {
    /// Creates a new form with the specified boundary generator function.
    ///
    /// # Examples
    ///
    /// ```
    /// # use multipart_rfc7578::Form;
    /// # use multipart_rfc7578::BoundaryGenerator;
    /// #
    /// struct TestGenerator;
    ///
    /// impl BoundaryGenerator for TestGenerator {
    ///     fn generate_boundary() -> String {
    ///         "test".to_string()
    ///     }
    /// }
    ///
    /// let form = Form::new::<TestGenerator>();
    /// ```
    ///
    #[inline]
    pub fn new<G>() -> Self
    where
        G: BoundaryGenerator,
    {
        Self {
            parts: vec![],
            boundary: G::generate_boundary(),
        }
    }

    /// Adds a text part to the Form.
    ///
    /// # Examples
    ///
    /// ```
    /// use multipart_rfc7578::Form;
    ///
    /// let mut form = Form::default();
    ///
    /// form.add_text("text", "Hello World!");
    /// form.add_text("more", String::from("Hello Universe!"));
    /// ```
    ///
    pub fn add_text<N, T>(&mut self, name: N, text: T)
    where
        N: Display,
        T: Into<String>,
    {
        self.parts.push(Part::new::<_, String>(
            Inner::Text(text.into()),
            name,
            None,
            None,
        ))
    }

    /// Adds a readable part to the Form.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate mime;
    /// # extern crate multipart_rfc7578;
    /// #
    /// use multipart_rfc7578::Form;
    /// use std::io::Cursor;
    ///
    /// let string = "Hello World!";
    /// let bytes = Cursor::new(string);
    /// let mut form = Form::default();
    ///
    /// form.add_reader2("input", bytes, Some("filename.png"), Some(mime::TEXT_PLAIN), Some(string.len() as u64));
    /// ```
    pub fn add_reader2<F, G, R>(
        &mut self,
        name: F,
        read: R,
        filename: Option<G>,
        mime: Option<Mime>,
        length: Option<u64>,
    ) where
        F: Display,
        G: Into<String>,
        R: 'static + Read + Send,
    {
        let read = Box::new(read);

        self.parts.push(Part::new::<_, String>(
            Inner::Read(read, length),
            name,
            mime,
            filename.map(Into::into),
        ));
    }

    /// Adds a readable part to the Form.
    ///
    /// # Examples
    ///
    /// ```
    /// use multipart_rfc7578::Form;
    /// use std::io::Cursor;
    ///
    /// let bytes = Cursor::new("Hello World!");
    /// let mut form = Form::default();
    ///
    /// form.add_reader("input", bytes);
    /// ```
    #[inline]
    pub fn add_reader<F, R>(&mut self, name: F, read: R)
    where
        F: Display,
        R: 'static + Read + Send,
    {
        self.add_reader2(name, read, None::<&str>, None, None);
    }

    /// Adds a readable part to the Form as a file.
    ///
    /// # Examples
    ///
    /// ```
    /// use multipart_rfc7578::Form;
    /// use std::io::Cursor;
    ///
    /// let bytes = Cursor::new("Hello World!");
    /// let mut form = Form::default();
    ///
    /// form.add_reader_file("input", bytes, "filename.txt");
    /// ```
    #[inline]
    pub fn add_reader_file<F, G, R>(&mut self, name: F, read: R, filename: G)
    where
        F: Display,
        G: Into<String>,
        R: 'static + Read + Send,
    {
        self.add_reader2(name, read, Some(filename), None, None);
    }

    /// Adds a readable part to the Form as a file with a specified mime.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate mime;
    /// # extern crate multipart_rfc7578;
    /// #
    /// use multipart_rfc7578::Form;
    /// use std::io::Cursor;
    ///
    /// # fn main() {
    /// let bytes = Cursor::new("Hello World!");
    /// let mut form = Form::default();
    ///
    /// form.add_reader_file_with_mime("input", bytes, "filename.txt", mime::TEXT_PLAIN);
    /// # }
    /// ```
    ///
    #[inline]
    pub fn add_reader_file_with_mime<F, G, R>(&mut self, name: F, read: R, filename: G, mime: Mime)
    where
        F: Display,
        G: Into<String>,
        R: 'static + Read + Send,
    {
        self.add_reader2(name, read, Some(filename), Some(mime), None);
    }

    /// Adds a file, and attempts to derive the mime type.
    ///
    /// # Examples
    ///
    /// ```
    /// use multipart_rfc7578::Form;
    ///
    /// let mut form = Form::default();
    ///
    /// form.add_file("file", file!()).expect("file to exist");
    /// ```
    ///
    #[inline]
    pub fn add_file<P, F>(&mut self, name: F, path: P) -> io::Result<()>
    where
        P: AsRef<Path>,
        F: Display,
    {
        self._add_file(name, path, None)
    }

    /// Adds a file with the specified mime type to the form.
    /// If the mime type isn't specified, a mime type will try to
    /// be derived.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate mime;
    /// # extern crate multipart_rfc7578;
    /// #
    /// use multipart_rfc7578::Form;
    ///
    /// # fn main() {
    /// let mut form = Form::default();
    ///
    /// form.add_file_with_mime("data", "test.csv", mime::TEXT_CSV);
    /// # }
    /// ```
    ///
    #[inline]
    pub fn add_file_with_mime<P, F>(&mut self, name: F, path: P, mime: Mime) -> io::Result<()>
    where
        P: AsRef<Path>,
        F: Display,
    {
        self._add_file(name, path, Some(mime))
    }

    /// Internal method for adding a file part to the form.
    ///
    fn _add_file<P, F>(&mut self, name: F, path: P, mime: Option<Mime>) -> io::Result<()>
    where
        P: AsRef<Path>,
        F: Display,
    {
        let f = File::open(&path)?;
        let mime = match mime {
            Some(mime) => Some(mime),
            None => match path.as_ref().extension() {
                Some(ext) => Mime::from_str(ext.to_string_lossy().borrow()).ok(),
                None => None,
            },
        };
        let len = match f.metadata() {
            // If the path is not a file, it can't be uploaded because there
            // is no content.
            //
            Ok(ref meta) if !meta.is_file() => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "expected a file not directory",
            )),

            // If there is some metadata on the file, try to derive some
            // header values.
            //
            Ok(ref meta) => Ok(Some(meta.len())),

            // The file metadata could not be accessed. This MIGHT not be an
            // error, if the file could be opened.
            //
            Err(e) => Err(e),
        }?;

        let read = Box::new(f);

        self.parts.push(Part::new(
            Inner::Read(read, len),
            name,
            mime,
            Some(path.as_ref().as_os_str().to_string_lossy()),
        ));

        Ok(())
    }

    /// get boundary as content type string
    #[inline]
    pub fn content_type(&self) -> String {
        format!("multipart/form-data; boundary=\"{}\"", &self.boundary)
    }

    #[inline]
    fn boundary_string(&self) -> String {
        format!("--{}{}", self.boundary, CRLF)
    }

    #[inline]
    fn final_boundary_string(&self) -> String {
        format!("--{}--{}", self.boundary, CRLF)
    }

    pub fn into_reader(self) -> impl Read {
        let boundary = Cursor::new(self.boundary_string());
        let final_boundary = Cursor::new(self.final_boundary_string());
        let readers = self
            .parts
            .into_iter()
            .map(|part| part.into_reader())
            .peekable();
        FormReader::new(boundary, readers, final_boundary)
    }

    #[inline]
    fn boundary_len(&self) -> u64 {
        (self.boundary.len() + 4) as u64
    }

    /// get content length
    pub fn content_length(&self) -> Option<u64> {
        let boundary_len = self.boundary_len() + 2;
        self.parts.iter().try_fold(boundary_len, |sum, part| {
            part.content_length().map(|len| sum + len + boundary_len)
        })
    }

    #[cfg(feature = "hyper")]
    /// Updates a request instance with the multipart Content-Type header
    /// and the payload data.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate hyper;
    /// # extern crate multipart_rfc7578;
    /// #
    /// use hyper::{Method, Request, Uri};
    /// use multipart_rfc7578::Form;
    ///
    /// # fn main() {
    /// let url: Uri = "http://localhost:80/upload".parse().unwrap();
    /// let mut req_builder = Request::post(url);
    /// let mut form = Form::default();
    ///
    /// form.add_text("text", "Hello World!");
    /// let req = form.set_hyper_body(&mut req_builder).unwrap();
    /// # }
    /// ```
    ///
    pub fn set_hyper_body(self, req: &mut Builder) -> Result<Request<hyper::Body>, http::Error> {
        req.header(CONTENT_TYPE, self.content_type());
        if let Some(len) = self.content_length() {
            req.header(CONTENT_LENGTH, len.to_string());
        }
        req.body(hyper::Body::wrap_stream(Body::from(self)))
    }

    #[cfg(feature = "actix-web")]
    /// Updates a request instance with the multipart Content-Type header
    /// and the payload data.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate actix_web;
    /// # extern crate multipart_rfc7578;
    /// #
    /// use actix_web::client;
    /// use multipart_rfc7578::Form;
    ///
    /// # fn main() {
    /// let url = "http://localhost:80/upload";
    /// let mut req_builder = client::post(url);
    /// let mut form = Form::default();
    ///
    /// form.add_text("text", "Hello World!");
    /// let req = form.set_actix_body(&mut req_builder).unwrap();
    /// # }
    /// ```
    ///
    pub fn set_actix_body(
        self,
        req: &mut ClientRequestBuilder,
    ) -> Result<ClientRequest, actix_web::Error> {
        req.header(CONTENT_TYPE, self.content_type());
        if let Some(len) = self.content_length() {
            req.header(CONTENT_LENGTH, len.to_string());
        }
        req.streaming(Body::from(self))
    }
}

#[cfg(test)]
mod tests {
    use super::Form;
    use std::io::{Cursor, Read};
    #[test]
    fn test_text_form() {
        let mut form = Form::default();
        form.add_text("hello", "world");
        form.add_text("foo", "bar");
        let test_string = format!(
            "--{}\r
content-disposition: form-data; name=\"hello\"\r
content-type: text/plain\r
\r
world\r
--{}\r
content-disposition: form-data; name=\"foo\"\r
content-type: text/plain\r
\r
bar\r
--{}--\r
",
            form.boundary, form.boundary, form.boundary
        );
        let mut form_string = String::with_capacity(test_string.len() + 1);
        form.into_reader().read_to_string(&mut form_string).unwrap();
        assert_eq!(test_string, form_string);
    }

    #[test]
    fn test_form_reader() {
        let mut form = Form::default();
        form.add_reader("hello", Cursor::new("world"));
        form.add_text("foo", "bar");
        let test_string = format!(
            "--{}\r
content-disposition: form-data; name=\"hello\"\r
content-type: application/octet-stream\r
\r
world\r
--{}\r
content-disposition: form-data; name=\"foo\"\r
content-type: text/plain\r
\r
bar\r
--{}--\r
",
            form.boundary, form.boundary, form.boundary
        );
        let test_string = test_string.to_string();
        let mut form_string = String::with_capacity(test_string.len() + 1);
        form.into_reader().read_to_string(&mut form_string).unwrap();
        assert_eq!(test_string, form_string);
    }
}
