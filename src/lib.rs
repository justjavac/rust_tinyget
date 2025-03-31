//! # tinyget
//! Simple, minimal-dependency HTTP client.
//! The library has a very minimal API, so you'll probably know
//! everything you need to after reading a few examples.
//!
//! # Additional features
//!
//! Since the crate is supposed to be minimal in terms of
//! dependencies, there are no default features, and optional
//! functionality can be enabled by specifying features for `tinyget`
//! dependency in `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! tinyget = { version = "1.0", features = ["https"] }
//! ```
//!
//! Below is the list of all available features.
//!
//! ## `https`
//!
//! This feature uses the (very good)
//! [`tls-native`](https://crates.io/crates/native-tls) crate to secure the
//! connection when needed. Note that if this feature is not enabled
//! (and it is not by default), requests to urls that start with
//! `https://` will fail and return a
//! [`HttpsFeatureNotEnabled`](enum.Error.html#variant.HttpsFeatureNotEnabled)
//! error.
//!
//! [`Request`](struct.Request.html) and
//! [`Response`](struct.Response.html) expose
//!
//! # Examples
//!
//! This is a simple example of sending a GET request and printing out
//! the response's body, status code, and reason phrase. The `?` are
//! needed because the server could return invalid UTF-8 in the body,
//! or something could go wrong during the download.
//!
//! ```
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let response = tinyget::get("http://httpbin.org/ip").send()?;
//! assert!(response.as_str()?.contains("\"origin\":"));
//! assert_eq!(response.status_code, 200);
//! assert_eq!(response.reason_phrase, "OK");
//! # Ok(()) }
//! ```
//!
//! ## Headers (sending)
//!
//! To add a header, add `with_header("Key", "Value")` before
//! `send()`.
//!
//! ```
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let response = tinyget::get("http://httpbin.org/headers")
//!     .with_header("Accept", "text/plain")
//!     .with_header("X-Best-Mon", "Sylveon")
//!     .send()?;
//! let body_str = response.as_str()?;
//! assert!(body_str.contains("\"Accept\": \"text/plain\""));
//! assert!(body_str.contains("\"X-Best-Mon\": \"Sylveon\""));
//! # Ok(()) }
//! ```
//!
//! ## Headers (receiving)
//!
//! Reading the headers sent by the servers is done via the
//! [`headers`](struct.Response.html#structfield.headers) field of the
//! [`Response`](struct.Response.html). Note: the header field names
//! (that is, the *keys* of the `HashMap`) are all lowercase: this is
//! because the names are case-insensitive according to the spec, and
//! this unifies the casings for easier `get()`ing.
//!
//! ```
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let response = tinyget::get("http://httpbin.org/ip").send()?;
//! assert_eq!(response.headers.get("content-type").unwrap(), "application/json");
//! # Ok(()) }
//! ```
//! 
//! ## Query Parameters
//!
//! To add query parameters to your request, use `with_query("key", "value")` before
//! `send()`.
//!
//! ```
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let response = tinyget::get("http://httpbin.org/get")
//!     .with_query("name", "John")
//!     .with_query("age", "30")
//!     .send()?;
//! let body_str = response.as_str()?;
//! assert!(body_str.contains("\"name\": \"John\""));
//! assert!(body_str.contains("\"age\": \"30\""));
//! # Ok(()) }
//! ```
//! 
//! ## Timeouts
//! To avoid timing out, or limit the request's response time, use
//! `with_timeout(n)` before `send()`. The given value is in seconds.
//!
//! NOTE: There is no timeout by default.
//! ```no_run
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let response = tinyget::get("http://httpbin.org/delay/6")
//!     .with_timeout(10)
//!     .send()?;
//! println!("{}", response.as_str()?);
//! # Ok(()) }
//! ```
//!
//! # Timeouts
//! By default, a request has no timeout.  You can change this in two ways:
//! - Use [`with_timeout`](struct.Request.html#method.with_timeout) on
//!   your request to set the timeout per-request like so:
//!   ```
//!   tinyget::get("/").with_timeout(8).send();
//!   ```
//! - Set the environment variable `TINYGET_TIMEOUT` to the desired
//!   amount of seconds until timeout. Ie. if you have a program called
//!   `foo` that uses tinyget, and you want all the requests made by that
//!   program to timeout in 8 seconds, you launch the program like so:
//!   ```text,ignore
//!   $ TINYGET_TIMEOUT=8 ./foo
//!   ```
//!   Or add the following somewhere before the requests in the code.
//!   ```
//!   std::env::set_var("TINYGET_TIMEOUT", "8");
//!   ```
//! If the timeout is set with `with_timeout`, the environment
//! variable will be ignored.

#![deny(missing_docs)]

#[cfg(feature = "https")]
extern crate native_tls;

mod connection;
mod error;
mod request;
mod response;

pub use error::*;
pub use request::*;
pub use response::*;
