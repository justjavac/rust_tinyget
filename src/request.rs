use crate::connection::Connection;
use crate::{Error, Response, ResponseLazy};
use std::collections::HashMap;

/// A URL type for requests.
pub type URL = String;

/// An HTTP request.
///
/// Generally created by the [`tinyget::get`](fn.get.html)-style
/// functions, corresponding to the HTTP method we want to use.
///
/// # Example
///
/// ```
/// let request = tinyget::get("http://example.com");
/// ```
///
/// After creating the request, you would generally call
/// [`send`](struct.Request.html#method.send) or
/// [`send_lazy`](struct.Request.html#method.send_lazy) on it, as it
/// doesn't do much on its own.
#[derive(Clone, PartialEq, Debug)]
pub struct Request {
    pub(crate) host: URL,
    resource: URL,
    headers: HashMap<String, String>,
    pub(crate) timeout: Option<u64>,
    max_redirects: usize,
    https: bool,
    pub(crate) redirects: Vec<(bool, URL, URL)>,
}

impl Request {
    /// Creates a new HTTP `Request`.
    ///
    /// This is only the request's data, it is not sent yet. For
    /// sending the request, see [`send`](struct.Request.html#method.send).
    pub fn new<T: Into<URL>>(url: T) -> Request {
        let (https, host, resource) = parse_url(url.into());
        Request {
            host,
            resource,
            headers: HashMap::new(),
            timeout: None,
            max_redirects: 100,
            https,
            redirects: Vec::new(),
        }
    }

    /// Adds a header to the request this is called on. Use this
    /// function to add headers to your requests.
    pub fn with_header<T: Into<String>, U: Into<String>>(mut self, key: T, value: U) -> Request {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Sets the request timeout in seconds.
    pub fn with_timeout(mut self, timeout: u64) -> Request {
        self.timeout = Some(timeout);
        self
    }

    /// Sets the max redirects we follow until giving up. 100 by
    /// default.
    ///
    /// Warning: setting this to a very high number, such as 1000, may
    /// cause a stack overflow if that many redirects are followed. If
    /// you have a use for so many redirects that the stack overflow
    /// becomes a problem, please open an issue.
    pub fn with_max_redirects(mut self, max_redirects: usize) -> Request {
        self.max_redirects = max_redirects;
        self
    }

    /// Sends this request to the host.
    ///
    /// # Errors
    ///
    /// Returns `Err` if we run into an error while sending the
    /// request, or receiving/parsing the response. The specific error
    /// is described in the `Err`, and it can be any
    /// [`tinyget::Error`](enum.Error.html) except
    /// [`InvalidUtf8InBody`](enum.Error.html#variant.InvalidUtf8InBody).
    #[cfg(feature = "https")]
    pub fn send(self) -> Result<Response, Error> {
        if self.https {
            let response = Connection::new(self).send_https()?;
            Response::create(response)
        } else {
            let response = Connection::new(self).send()?;
            Response::create(response)
        }
    }

    /// Sends this request to the host, loaded lazily.
    ///
    /// # Errors
    ///
    /// See [`send`](struct.Request.html#method.send).
    #[cfg(feature = "https")]
    pub fn send_lazy(self) -> Result<ResponseLazy, Error> {
        if self.https {
            Connection::new(self).send_https()
        } else {
            Connection::new(self).send()
        }
    }

    /// Sends this request to the host.
    ///
    /// # Errors
    ///
    /// Returns `Err` if we run into an error while sending the
    /// request, or receiving/parsing the response. The specific error
    /// is described in the `Err`, and it can be any
    /// [`tinyget::Error`](enum.Error.html) except
    /// [`InvalidUtf8InBody`](enum.Error.html#variant.InvalidUtf8InBody).
    #[cfg(not(feature = "https"))]
    pub fn send(self) -> Result<Response, Error> {
        if self.https {
            Err(Error::HttpsFeatureNotEnabled)
        } else {
            let response = Connection::new(self).send()?;
            Response::create(response)
        }
    }

    /// Sends this request to the host, loaded lazily.
    ///
    /// # Errors
    ///
    /// See [`send`](struct.Request.html#method.send).
    #[cfg(not(feature = "https"))]
    pub fn send_lazy(self) -> Result<ResponseLazy, Error> {
        if self.https {
            Err(Error::HttpsFeatureNotEnabled)
        } else {
            Connection::new(self).send()
        }
    }

    /// Returns the HTTP request as bytes, ready to be sent to
    /// the server.
    pub(crate) fn as_bytes(&self) -> Vec<u8> {
        let mut http = String::with_capacity(32);
        // Add the request line and the "Host" header
        http += &format!("GET {} HTTP/1.1\r\nHost: {}\r\n", self.resource, self.host);
        // Add other headers
        for (k, v) in &self.headers {
            http += &format!("{}: {}\r\n", k, v);
        }

        http += "\r\n";
        http.into_bytes()
    }

    /// Returns the redirected version of this Request, unless an
    /// infinite redirection loop was detected, or the redirection
    /// limit was reached.
    pub(crate) fn redirect_to(mut self, url: URL) -> Result<Request, Error> {
        // If the redirected resource does not have a fragment, but
        // the original URL did, the fragment should be preserved over
        // redirections. See RFC 7231 section 7.1.2.
        let inherit_fragment = |resource: String, original_resource: &str| {
            if resource.chars().any(|c| c == '#') {
                println!("Resource has a resource, not inheriting.");
                resource
            } else {
                let mut original_resource_split = original_resource.split('#');
                if let Some(fragment) = original_resource_split.nth(1) {
                    println!("Using inherited fragment.");
                    format!("{}#{}", resource, fragment)
                } else {
                    println!(
                        "Neither has a resource, not inheriting. Original: {}",
                        original_resource
                    );
                    resource
                }
            }
        };

        if url.contains("://") {
            let (https, host, resource) = parse_url(url);
            let new_resource = inherit_fragment(resource, &self.resource);

            self.redirects.push((self.https, self.host, self.resource));

            self.https = https;
            self.resource = new_resource;
            self.host = host;
        } else {
            // The url does not have the protocol part, assuming it's
            // a relative resource.
            let new_resource = inherit_fragment(url, &self.resource);

            self.redirects
                .push((self.https, self.host.clone(), self.resource));

            self.resource = new_resource;
        }

        let is_this_url = |(https_, host_, resource_): &(bool, URL, URL)| {
            resource_ == &self.resource && host_ == &self.host && https_ == &self.https
        };

        if self.redirects.len() > self.max_redirects {
            Err(Error::TooManyRedirections)
        } else if self.redirects.iter().any(is_this_url) {
            Err(Error::InfiniteRedirectionLoop)
        } else {
            Ok(self)
        }
    }
}

fn parse_url(url: URL) -> (bool, URL, URL) {
    let mut first = URL::new();
    let mut second = URL::new();
    let mut slashes = 0;
    for c in url.chars() {
        if c == '/' {
            slashes += 1;
        } else if slashes == 2 {
            first.push(c);
        }
        if slashes >= 3 {
            second.push(c);
        }
    }
    // Ensure the resource is *something*
    if second.is_empty() {
        second += "/";
    }
    // Set appropriate port
    let https = url.starts_with("https://");
    if !first.contains(':') {
        first += if https { ":443" } else { ":80" };
    }
    (https, first, second)
}

/// Alias for [Request::new](struct.Request.html#method.new)
pub fn get<T: Into<URL>>(url: T) -> Request {
    Request::new(url)
}
