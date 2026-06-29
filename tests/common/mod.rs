extern crate tiny_http;
extern crate tinyget;
use self::tiny_http::{Header, Method, Response, Server};
use std::sync::Arc;
use std::sync::Once;
use std::thread;
use std::time::Duration;

static INIT: Once = Once::new();

pub fn setup() {
    INIT.call_once(|| {
        let server = Arc::new(Server::http("localhost:35562").unwrap());
        for _ in 0..4 {
            let server = server.clone();

            thread::spawn(move || loop {
                let mut request = {
                    if let Ok(request) = server.recv() {
                        request
                    } else {
                        continue; // If .recv() fails, just try again.
                    }
                };
                let content = "Q";
                let headers = Vec::from(request.headers());
                let fragment = request.url().split('#').nth(1).unwrap_or("");

                let url = String::from(request.url().split('#').next().unwrap());
                match request.method() {
                    Method::Get if url == "/header_pong" => {
                        for header in headers {
                            if header.field.as_str() == "Ping" {
                                let response = Response::from_string(format!("{}", header.value));
                                request.respond(response).ok();
                                return;
                            }
                        }
                        request.respond(Response::from_string("No header!")).ok();
                    }

                    Method::Get if url.starts_with("/query") => {
                        let response = Response::from_string(echo_query_params(&url));
                        request.respond(response).ok();
                    }

                    Method::Post if url == "/post" => {
                        let mut body = String::new();
                        request.as_reader().read_to_string(&mut body).ok();
                        let response =
                            Response::from_string(format!("method: POST\nbody: {}", body));
                        request.respond(response).ok();
                    }

                    Method::Get if url == "/slow_a" => {
                        thread::sleep(Duration::from_secs(2));
                        let response = Response::from_string(format!("j: {}", content));
                        request.respond(response).ok();
                    }

                    Method::Get if url == "/a" => {
                        let response = Response::from_string(format!("j: {}{}", content, fragment));
                        request.respond(response).ok();
                    }

                    Method::Get if url == "/redirect-baz" => {
                        let response = Response::empty(301).with_header(
                            Header::from_bytes(
                                &b"Location"[..],
                                &b"http://localhost:35562/a#baz"[..],
                            )
                            .unwrap(),
                        );
                        request.respond(response).ok();
                    }

                    Method::Get if url == "/redirect" => {
                        let response = Response::empty(301).with_header(
                            Header::from_bytes(&b"Location"[..], &b"http://localhost:35562/a"[..])
                                .unwrap(),
                        );
                        request.respond(response).ok();
                    }

                    Method::Get if url == "/infiniteredirect" => {
                        let response = Response::empty(301).with_header(
                            Header::from_bytes(
                                &b"Location"[..],
                                &b"http://localhost:35562/redirectpong"[..],
                            )
                            .unwrap(),
                        );
                        request.respond(response).ok();
                    }

                    Method::Get if url == "/redirectpong" => {
                        let response = Response::empty(301).with_header(
                            Header::from_bytes(
                                &b"Location"[..],
                                &b"http://localhost:35562/infiniteredirect"[..],
                            )
                            .unwrap(),
                        );
                        request.respond(response).ok();
                    }

                    Method::Get if url == "/relativeredirect" => {
                        let response = Response::empty(303)
                            .with_header(Header::from_bytes(&b"Location"[..], &b"/a"[..]).unwrap());
                        request.respond(response).ok();
                    }

                    _ => {
                        request
                            .respond(Response::from_string("Not Found").with_status_code(404))
                            .ok();
                    }
                }
            });
        }
    });
}

pub fn url(req: &str) -> String {
    format!("http://localhost:35562{}", req)
}

pub fn get_body(request: Result<tinyget::Response, tinyget::Error>) -> String {
    match request {
        Ok(response) => match response.as_str() {
            Ok(str) => String::from(str),
            Err(err) => {
                println!("\n[ERROR]: {}\n", err);
                String::new()
            }
        },
        Err(err) => {
            println!("\n[ERROR]: {}\n", err);
            String::new()
        }
    }
}

pub fn get_status_code(request: Result<tinyget::Response, tinyget::Error>) -> i32 {
    match request {
        Ok(response) => response.status_code,
        Err(err) => {
            println!("\n[ERROR]: {}\n", err);
            -1
        }
    }
}

fn echo_query_params(url: &str) -> String {
    let query = url.split_once('?').map(|(_, query)| query).unwrap_or("");
    let mut body = String::new();

    for pair in query.split('&').filter(|pair| !pair.is_empty()) {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        let key = decode_query_component(key);
        let value = decode_query_component(value);
        body.push_str(&format!(
            "\"{}\": \"{}\"\n",
            escape_json(&key),
            escape_json(&value)
        ));
    }

    body
}

fn decode_query_component(value: &str) -> String {
    urlencoding::decode(value)
        .map(|value| value.into_owned())
        .unwrap_or_else(|_| value.to_string())
}

fn escape_json(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
