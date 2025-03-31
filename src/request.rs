use crate::connection::Connection;
use crate::{Error, Response, ResponseLazy};
use std::collections::HashMap;
use urlencoding;

/// A URL type for requests.
#[allow(clippy::upper_case_acronyms)]
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
    query: HashMap<String, String>,
    #[cfg(feature = "timeout")]
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
            query: HashMap::new(),
            #[cfg(feature = "timeout")]
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

    /// Adds a query parameter to the URL.
    pub fn with_query<T: Into<String>, U: Into<String>>(mut self, key: T, value: U) -> Request {
        self.query.insert(key.into(), value.into());
        self
    }

    /// Sets the request timeout in seconds.
    #[cfg(feature = "timeout")]
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
            #[cfg(feature = "timeout")]
            {
                let response = match self.timeout {
                    Some(timeout) => Connection::new(self)
                        .send_https_timeout(std::time::Duration::from_secs(timeout))?,
                    None => Connection::new(self).send_https()?,
                };
                Response::create(response)
            }

            #[cfg(not(feature = "timeout"))]
            {
                let response = Connection::new(self).send_https()?;
                Response::create(response)
            }
        } else {
            #[cfg(feature = "timeout")]
            {
                let response = match self.timeout {
                    Some(timeout) => Connection::new(self)
                        .send_timeout(std::time::Duration::from_secs(timeout))?,
                    None => Connection::new(self).send()?,
                };
                Response::create(response)
            }

            #[cfg(not(feature = "timeout"))]
            {
                let response = Connection::new(self).send()?;
                Response::create(response)
            }
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
            #[cfg(feature = "timeout")]
            {
                let response = match self.timeout {
                    Some(timeout) => Connection::new(self)
                        .send_timeout(std::time::Duration::from_secs(timeout))?,
                    None => Connection::new(self).send()?,
                };
                Response::create(response)
            }

            #[cfg(not(feature = "timeout"))]
            {
                let response = Connection::new(self).send()?;
                Response::create(response)
            }
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
        let resource = if self.query.is_empty() {
            self.resource.clone()
        } else {
            let mut query_string = String::new();
            for (i, (k, v)) in self.query.iter().enumerate() {
                // Check if original URL already contains query parameters
                let separator = if i == 0 && !self.resource.contains('?') {
                    '?'
                } else {
                    '&'
                };
                query_string.push(separator);
                query_string.push_str(&urlencoding::encode(k));
                query_string.push('=');
                query_string.push_str(&urlencoding::encode(v));
            }
            format!("{}{}", self.resource, query_string)
        };
        http += &format!("GET {} HTTP/1.1\r\nHost: {}\r\n", resource, self.host);
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
                resource
            } else {
                let mut original_resource_split = original_resource.split('#');
                if let Some(fragment) = original_resource_split.nth(1) {
                    format!("{}#{}", resource, fragment)
                } else {
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
