use crate::{Error, Request, ResponseLazy};
#[cfg(feature = "https")]
use native_tls::{TlsConnector, TlsStream};
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::net::TcpStream;
#[cfg(feature = "timeout")]
use std::net::ToSocketAddrs;
#[cfg(feature = "timeout")]
use std::time::Duration;
use std::time::Instant;

type UnsecuredStream = BufReader<TcpStream>;
#[cfg(feature = "https")]
type SecuredStream = TlsStream<TcpStream>;

pub(crate) enum HttpStream {
    Unsecured(UnsecuredStream, Option<Instant>),
    #[cfg(feature = "https")]
    Secured(Box<SecuredStream>, Option<Instant>),
}

impl HttpStream {
    fn create_unsecured(reader: UnsecuredStream, timeout_at: Option<Instant>) -> HttpStream {
        HttpStream::Unsecured(reader, timeout_at)
    }

    #[cfg(feature = "https")]
    fn create_secured(reader: SecuredStream, timeout_at: Option<Instant>) -> HttpStream {
        HttpStream::Secured(Box::new(reader), timeout_at)
    }
}

impl Read for HttpStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let timeout = |tcp: &TcpStream, timeout_at: Option<Instant>| {
            if let Some(timeout_at) = timeout_at {
                let now = Instant::now();
                if timeout_at <= now {
                    return Err(io::Error::new(
                        io::ErrorKind::TimedOut,
                        "The request's timeout was reached.",
                    ));
                } else {
                    tcp.set_read_timeout(Some(timeout_at - now)).ok();
                }
            }
            Ok(())
        };

        match self {
            HttpStream::Unsecured(inner, timeout_at) => {
                timeout(inner.get_ref(), *timeout_at)?;
                inner.read(buf)
            }
            #[cfg(feature = "https")]
            HttpStream::Secured(inner, timeout_at) => {
                timeout(inner.get_ref(), *timeout_at)?;
                inner.read(buf)
            }
        }
    }
}

/// A connection to the server for sending
/// [`Request`](struct.Request.html)s.
pub struct Connection {
    request: Request,
    #[cfg(feature = "timeout")]
    timeout: Option<u64>,
}

impl Connection {
    /// Creates a new `Connection`. See
    /// [`Request`](struct.Request.html) for specifics about *what* is
    /// being sent.
    pub(crate) fn new(request: Request) -> Connection {
        #[cfg(feature = "timeout")]
        {
            let timeout = request
                .timeout
                .or_else(|| match std::env::var("TINYGET_TIMEOUT") {
                    Ok(t) => t.parse::<u64>().ok(),
                    Err(_) => None,
                });
            Connection { request, timeout }
        }
        #[cfg(not(feature = "timeout"))]
        {
            Connection { request }
        }
    }

    /// Sends the [`Request`](struct.Request.html), consumes this
    /// connection, and returns a [`Response`](struct.Response.html).
    #[cfg(feature = "https")]
    pub(crate) fn send_https(self) -> Result<ResponseLazy, Error> {
        let bytes = self.request.as_bytes();

        let dns_name = &self.request.host;
        // parse_url in response.rs ensures that there is always a
        // ":port" in the host, which is why this unwrap is safe.
        let dns_name = dns_name.split(':').next().unwrap();
        /*
        let mut builder = TlsConnector::builder();
        ...
        let sess = match builder.build() {
        */
        let sess = match TlsConnector::new() {
            Ok(sess) => sess,
            Err(err) => return Err(Error::IoError(io::Error::new(io::ErrorKind::Other, err))),
        };

        let tcp = self.connect()?;

        // Send request
        let mut tls = match sess.connect(dns_name, tcp) {
            Ok(tls) => tls,
            Err(err) => return Err(Error::IoError(io::Error::new(io::ErrorKind::Other, err))),
        };
        // The connection could drop mid-write, so set a timeout
        tls.write(&bytes)?;

        // Receive request
        let response = ResponseLazy::from_stream(HttpStream::create_secured(tls, None))?;
        handle_redirects(self, response)
    }

    /// Sends the [`Request`](struct.Request.html), consumes this
    /// connection, and returns a [`Response`](struct.Response.html).
    #[cfg(all(feature = "https", feature = "timeout"))]
    pub(crate) fn send_https_timeout(self, timeout: Duration) -> Result<ResponseLazy, Error> {
        let bytes = self.request.as_bytes();
        let timeout_duration = self.timeout.map(Duration::from_secs);
        let timeout_at = timeout_duration.map(|d| Instant::now() + d);

        let dns_name = &self.request.host;
        // parse_url in response.rs ensures that there is always a
        // ":port" in the host, which is why this unwrap is safe.
        let dns_name = dns_name.split(':').next().unwrap();
        /*
        let mut builder = TlsConnector::builder();
        ...
        let sess = match builder.build() {
        */
        let sess = match TlsConnector::new() {
            Ok(sess) => sess,
            Err(err) => return Err(Error::IoError(io::Error::new(io::ErrorKind::Other, err))),
        };

        let tcp = self.connect_timeout(timeout)?;

        // Send request
        let mut tls = match sess.connect(dns_name, tcp) {
            Ok(tls) => tls,
            Err(err) => return Err(Error::IoError(io::Error::new(io::ErrorKind::Other, err))),
        };
        // The connection could drop mid-write, so set a timeout
        tls.write(&bytes)?;

        // Receive request
        let response = ResponseLazy::from_stream(HttpStream::create_secured(tls, timeout_at))?;
        handle_redirects(self, response)
    }

    /// Sends the [`Request`](struct.Request.html), consumes this
    /// connection, and returns a [`Response`](struct.Response.html).
    pub(crate) fn send(self) -> Result<ResponseLazy, Error> {
        let bytes = self.request.as_bytes();
        let tcp = self.connect()?;

        // Send request
        let mut stream = BufWriter::new(tcp);
        stream.write_all(&bytes)?;

        // Receive response
        let tcp = match stream.into_inner() {
            Ok(tcp) => tcp,
            Err(_) => {
                return Err(Error::Other(
                    "IntoInnerError after writing the request into the TcpStream.",
                ));
            }
        };
        let stream = HttpStream::create_unsecured(BufReader::new(tcp), None);
        let response = ResponseLazy::from_stream(stream)?;
        handle_redirects(self, response)
    }

    fn connect(&self) -> Result<TcpStream, Error> {
        TcpStream::connect(&self.request.host).map_err(Error::from)
    }

    /// Sends the [`Request`](struct.Request.html), consumes this
    /// connection, and returns a [`Response`](struct.Response.html).
    #[cfg(feature = "timeout")]
    #[allow(dead_code)]
    pub(crate) fn send_timeout(self, timeout: Duration) -> Result<ResponseLazy, Error> {
        let bytes = self.request.as_bytes();
        let timeout_duration = self.timeout.map(Duration::from_secs);
        let timeout_at = timeout_duration.map(|d| Instant::now() + d);

        let tcp = self.connect_timeout(timeout)?;

        // Send request
        let mut stream = BufWriter::new(tcp);
        stream.get_ref().set_write_timeout(timeout_duration).ok();
        stream.write_all(&bytes)?;

        // Receive response
        let tcp = match stream.into_inner() {
            Ok(tcp) => tcp,
            Err(_) => {
                return Err(Error::Other(
                    "IntoInnerError after writing the request into the TcpStream.",
                ));
            }
        };
        let stream = HttpStream::create_unsecured(BufReader::new(tcp), timeout_at);
        let response = ResponseLazy::from_stream(stream)?;
        handle_redirects(self, response)
    }

    #[cfg(feature = "timeout")]
    fn connect_timeout(&self, timeout: Duration) -> Result<TcpStream, Error> {
        let addr = self
            .request
            .host
            .to_socket_addrs()?
            .next()
            .ok_or(Error::Other("Failed to resolve host to SocketAddr"))?;
        TcpStream::connect_timeout(&addr, timeout).map_err(Error::from)
    }
}

fn handle_redirects(connection: Connection, response: ResponseLazy) -> Result<ResponseLazy, Error> {
    let status_code = response.status_code;
    let url = response.headers.get("location");
    if let Some(request) = get_redirect(connection, status_code, url) {
        request?.send_lazy()
    } else {
        Ok(response)
    }
}

fn get_redirect(
    connection: Connection,
    status_code: i32,
    url: Option<&String>,
) -> Option<Result<Request, Error>> {
    match status_code {
        301 | 302 | 303 | 307 => match url {
            Some(url) => Some(connection.request.redirect_to(url.clone())),
            None => Some(Err(Error::RedirectLocationMissing)),
        },

        _ => None,
    }
}
