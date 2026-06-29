extern crate tinyget;
mod common;

use self::common::*;

#[test]
fn test_basic_functionality() -> Result<(), Box<dyn std::error::Error>> {
    setup();
    let response = tinyget::get(url("/header_pong"))
        .with_header("Ping", "Test")
        .send()?;

    let body = get_body(Ok(response.clone()));
    let status_code = get_status_code(Ok(response));

    assert_eq!(status_code, 200);
    assert_eq!(body, "Test");
    Ok(())
}

#[test]
fn test_basic_query() -> Result<(), Box<dyn std::error::Error>> {
    setup();
    let response = tinyget::get(url("/query"))
        .with_query("name", "Tiny")
        .send()?;

    let body = get_body(Ok(response));

    assert!(body.contains("\"name\": \"Tiny\""));
    Ok(())
}

#[test]
fn test_multiple_queries() -> Result<(), Box<dyn std::error::Error>> {
    setup();
    let response = tinyget::get(url("/query"))
        .with_query("name", "Tiny")
        .with_query("age", "30")
        .send()?;

    let body = get_body(Ok(response));

    assert!(body.contains("\"name\": \"Tiny\""));
    assert!(body.contains("\"age\": \"30\""));
    Ok(())
}

#[test]
fn test_special_characters() -> Result<(), Box<dyn std::error::Error>> {
    setup();
    let response = tinyget::get(url("/query"))
        .with_query("message", "Hello World!")
        .with_query("user", "Tiny")
        .send()?;

    let body = get_body(Ok(response));

    assert!(body.contains("\"message\": \"Hello World!\""));
    assert!(body.contains("\"user\": \"Tiny\""));
    Ok(())
}

#[test]
fn test_chinese_characters() -> Result<(), Box<dyn std::error::Error>> {
    setup();
    let response = tinyget::get(url("/query"))
        .with_query("name", "张三")
        .send()?;

    let body = get_body(Ok(response));

    assert!(body.contains("\"name\": \"张三\""));
    Ok(())
}

#[test]
fn test_existing_query_parameters() -> Result<(), Box<dyn std::error::Error>> {
    setup();
    let response = tinyget::get(url("/query?existing=param"))
        .with_query("name", "Tiny")
        .with_query("age", "25")
        .send()?;

    let body = get_body(Ok(response));

    assert!(body.contains("\"existing\": \"param\""));
    assert!(body.contains("\"name\": \"Tiny\""));
    assert!(body.contains("\"age\": \"25\""));
    Ok(())
}
