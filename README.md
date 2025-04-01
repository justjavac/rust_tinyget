# tinyget

[![ci](https://github.com/justjavac/rust_tinyget/actions/workflows/ci.yml/badge.svg)](https://github.com/justjavac/rust_tinyget/actions/workflows/ci.yml)
[![Crate](https://img.shields.io/crates/v/tinyget.svg)](https://crates.io/crates/tinyget)
[![Documentation](https://docs.rs/tinyget/badge.svg)](https://docs.rs/tinyget)
![License](https://img.shields.io/crates/l/tinyget.svg)

> A tiny fork of [minreq](https://crates.io/crates/minreq).

A simple, minimal-dependency HTTP client for Rust. It provides a clean and intuitive API for making HTTP requests with minimal overhead.

## Features

- Simple and intuitive API
- Minimal dependencies
- Optional HTTPS support via native-tls
- Optional timeout support
- Small binary size

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
tinyget = "1.1"
```

Basic usage:

```rust
use tinyget;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Simple GET request
    let response = tinyget::get("https://httpbin.org/anything").send()?;
    println!("Response: {}", response.as_str()?);

    // With timeout
    let response = tinyget::get("https://httpbin.org/anything")
        .with_timeout(5)
        .send()?;
    println!("Response: {}", response.as_str()?);

    Ok(())
}
```

## Advanced Usage

### HTTPS Support

To enable HTTPS support, add the `https` feature:

```toml
[dependencies]
tinyget = { version = "1.1", features = ["https"] }
```

### Timeout Support

To enable timeout support, add the `timeout` feature:

```toml
[dependencies]
tinyget = { version = "1.1", features = ["timeout"] }
```

You can set timeout in two ways:

1. Per-request timeout:
```rust
let response = tinyget::get("https://httpbin.org/anything")
    .with_timeout(5)
    .send()?;
```

2. Global timeout via environment variable:
```bash
TINYGET_TIMEOUT=5 cargo run
```

### Custom Headers

```rust
let response = tinyget::get("https://httpbin.org/anything")
    .with_header("User-Agent", "tinyget/1.1")
    .send()?;
```

## Binary Size

rustc 1.76.0 (07dca489a 2024-02-04)

|                | debug              | release        |
| -------------- | ------------------ | -------------- |
| [**hello**][1] |   424,896          | 266,547        |
| [**http**][2]  |   772,416(+348k)   | 319,856(+53k)  |
| [**https**][3] | 1,101,512(+677k)   | 344,432(+78k)  |

[1]: ./examples/hello.rs
[2]: ./examples/http.rs
[3]: ./examples/https.rs

## Size Comparison

|             |      http |     https |
| ----------- | --------: | --------: |
| **tinyget** |   283,920 |   319,632 |
| **minreq**  |   300,328 |   959,744 |
| **ureq**    |   695,632 | 1,371,368 |
| **reqwest** | 1,639,496 | 1,675,032 |

## Examples

Check out the [examples](./examples) directory for more usage examples:

- [Basic HTTP request](./examples/http.rs)
- [HTTP request with timeout](./examples/http_timeout.rs)
- [HTTPS request](./examples/https.rs)
- [HTTPS request with timeout](./examples/https_timeout.rs)
- [Iterator example](./examples/iterator.rs)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This crate is distributed under the terms of the [MIT license](./LICENSE).
