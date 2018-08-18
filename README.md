## Rust Hyper Multipart (RFC 7578)

[![Travis](https://img.shields.io/travis/jeizsm/rust-multipart-rfc7578.svg)](https://travis-ci.org/jeizsm/rust-multipart-rfc7578)
[![Crates.io](https://img.shields.io/crates/v/multipart-rfc7578.svg)](https://crates.io/crates/multipart-rfc7578)
[![Docs.rs](https://docs.rs/multipart-rfc7578/badge.svg)](https://docs.rs/multipart-rfc7578/)

This crate contains an implementation of the multipart/form-data media
type described in [RFC 7578](https://tools.ietf.org/html/rfc7578) for
hyper.

Currently, only the client-side is implemented.

### Usage

```toml
[dependencies]
multipart-rfc7578 = "0.3"
```

Because the name of this library is really wordy, I recommend shortening it:

Using this requires a hyper client compatible with the `multipart::Body`
data structure (see the documentation for more detailed examples):

```rust

use hyper::{Client, Request, rt::{self, Future}};
use multipart_rfc7578::MultipartForm;

let client = Client::new();
let mut req_builder = Request::get("http://localhost/upload");
let mut form = MultipartForm::default();

form.add_text("test", "Hello World");
let req = form.set_body(&mut req_builder).unwrap();

rt::run(
    client
        .request(req)
        .map(|_| println!("done..."))
        .map_err(|_| println!("an error occurred")),
);
```


## Note on Server Implementation

I don't have any plans on implementing the server-side of this any time soon. I ended up implementing the client-side because I couldn't find any good libraries that were compatible with hyper >= 0.11.

Please feel free to submit a pull request, I would gladly review it!

## Alternatives

  * [abonander/multipart](https://github.com/abonander/multipart)
  * [abonander/multipart-async](https://crates.io/crates/multipart-async)
  * [mikedilger/formdata](https://github.com/mikedilger/formdata)

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
