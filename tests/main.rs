extern crate tinyget;
mod setup;

use self::setup::*;

#[test]
// Test based on issue #23: https://github.com/neonmoe/minreq/issues/23
fn test_headers_char_boundary_panic() {
    // This will panic with a `assertion failed: self.is_char_boundary(at)`
    // until the issue is fixed.
    tinyget::get("http://iheartradio.com").send().ok();
}

#[test]
#[cfg(feature = "https")]
// Test based on issue #24: https://github.com/neonmoe/minreq/issues/24
fn test_dns_name_error() {
    // This will panic by unwrapping a InvalidDNSNameError until the
    // issue is fixed.
    tinyget::get("http://virtualflorist.com").send().ok();
}

#[test]
#[cfg(feature = "https")]
fn test_https() {
    // TODO: Implement this locally.
    assert_eq!(
        get_status_code(tinyget::get("https://httpbin.org/status/418").send()),
        418
    );
}

#[test]
#[cfg(feature = "timeout")]
fn test_timeout_too_low() {
    setup();
    let result = tinyget::get(url("/slow_a")).with_timeout(1).send();
    assert!(result.is_err());
}

#[test]
#[cfg(feature = "timeout")]
fn test_timeout_high_enough() {
    setup();
    let body = get_body(tinyget::get(url("/slow_a")).with_timeout(3).send());
    assert_eq!(body, "j: Q");
}

#[test]
fn test_headers() {
    setup();
    let body = get_body(
        tinyget::get(url("/header_pong"))
            .with_header("Ping", "Qwerty")
            .send(),
    );
    assert_eq!("Qwerty", body);
}

#[test]
fn test_custom_method() {
    setup();
    let body = get_body(tinyget::Request::new(url("/a")).send());
    assert_eq!("j: Q", body);
}

#[test]
fn test_head() {
    setup();
    assert_eq!(get_status_code(tinyget::get(url("/a")).send()), 200);
}

#[test]
fn test_get() {
    setup();
    let body = get_body(tinyget::get(url("/a")).send());
    assert_eq!(body, "j: Q");
}

#[test]
fn test_redirect_get() {
    setup();
    let body = get_body(tinyget::get(url("/redirect")).send());
    assert_eq!(body, "j: Q");
}

#[test]
fn test_redirect_with_fragment() {
    setup();
    let body = get_body(tinyget::get(url("/redirect#foo")).send());
    assert_eq!(body, "j: Qfoo");
}

#[test]
fn test_redirect_with_overridden_fragment() {
    setup();
    let body = get_body(tinyget::get(url("/redirect-baz#foo")).send());
    assert_eq!(body, "j: Qbaz");
}

#[test]
fn test_infinite_redirect() {
    setup();
    let body = tinyget::get(url("/infiniteredirect")).send();
    assert!(body.is_err());
}

#[test]
fn test_relative_redirect_get() {
    setup();
    let body = get_body(tinyget::get(url("/relativeredirect")).send());
    assert_eq!(body, "j: Q");
}
