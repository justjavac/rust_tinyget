# tinyget

[![ci](https://github.com/justjavac/rust_tinyget/actions/workflows/ci.yml/badge.svg)](https://github.com/justjavac/rust_tinyget/actions/workflows/ci.yml)
[![Crate](https://img.shields.io/crates/v/tinyget.svg)](https://crates.io/crates/tinyget)
[![Documentation](https://docs.rs/tinyget/badge.svg)](https://docs.rs/tinyget)
![License](https://img.shields.io/crates/l/tinyget.svg)

> a tiny fork of [minreq](https://crates.io/crates/minreq).

Simple, minimal-dependency HTTP client. Optional features for https with `native-tls` TLS implementations.

[Documentation](https://docs.rs/tinyget)

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
tinyget = "1.0"
```

```rust
let response = tinyget::get("https://httpbin.org/anything").send()?;
let hello = response.as_str()?;
println!("{}", hello);
```

## Size

rustc 1.49.0 (e1884a8e3 2020-12-29) 

|                  | debug          | release       |
|------------------|----------------|---------------|
| [**hello**][1]   | 262,864        | 233,752       |
| [**http**][2]    | 498,040(+235k) | 283,920(+50k) |
| [**https**][3]   | 702,696(+440k) | 319,632(+87k) |

[1]: ./examples/hello.rs
[2]: ./examples/http.rs
[3]: ./examples/https.rs

## Compare

|                  | http       | https     |
|------------------|-----------:|----------:|
| **tinyget**      |    283,920 |   319,632 |
| **minreq**       |    300,328 |   959,744 |
| **ureq**         |    695,632 | 1,371,368 |
| **reqwest**      |  1,639,496 | 1,675,032 |

## License

This crate is distributed under the terms of the [MIT license](./LICENSE).
