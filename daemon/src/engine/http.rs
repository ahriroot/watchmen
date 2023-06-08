use std::{convert::Infallible, error::Error};

use common::config::Config;
use common::handle;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server};
use tokio::task::JoinHandle;
use tracing::{error, info};

use crate::command;

pub async fn start(config: Config) -> JoinHandle<()> {
    tokio::spawn(async move {
        match run_http(&config.http.host, config.http.port).await {
            Ok(_) => {
                info!("http server exit");
            }
            Err(e) => {
                error!("http server error: {}", e);
            }
        }
    })
}

pub async fn run_http(host: &str, port: u16) -> Result<(), Box<dyn Error>> {
    let addr: std::net::SocketAddr = format!("{}:{}", host, port).parse()?;
    info!("http server listen on {}", addr);
    let make_svc =
        make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle_connection)) });
    let server = Server::bind(&addr).serve(make_svc);
    if let Err(e) = server.await {
        error!("server error: {}", e);
    }
    Ok(())
}

async fn handle_connection(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    if req.method() == &hyper::Method::OPTIONS {
        let mut response = Response::new(Body::empty());
        let headers = response.headers_mut();
        headers.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
        headers.insert(
            "Access-Control-Allow-Methods",
            "GET, POST, PUT, DELETE".parse().unwrap(),
        );
        headers.insert(
            "Access-Control-Allow-Headers",
            "Content-Type".parse().unwrap(),
        );
        return Ok(response);
    }
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") | (&Method::GET, "/index.html") => {
            // return index.html
            let body = include_str!("../../http-panel/dist/index.html");
            let response = Response::builder()
                .header("Content-Type", "text/html")
                .body(Body::from(body))
                .unwrap();
            Ok(response)
        }
        (&Method::GET, "/favicon.svg") => {
            let body = include_str!("../../http-panel/dist/favicon.svg");
            let response = Response::builder()
                .header("Content-Type", "image/svg+xml")
                .body(Body::from(body))
                .unwrap();
            Ok(response)
        }
        (&Method::GET, "/index.css") => {
            // return index.html
            let body = include_str!("../../http-panel/dist/index.css");
            let response = Response::builder()
                .header("Content-Type", "text/css")
                .body(Body::from(body))
                .unwrap();
            Ok(response)
        }
        (&Method::GET, "/index.js") => {
            // return index.html
            let body = include_str!("../../http-panel/dist/index.js");
            let response = Response::builder()
                .header("Content-Type", "application/javascript")
                .body(Body::from(body))
                .unwrap();
            Ok(response)
        }
        (&Method::POST, "/api") => {
            let responses = match hyper::body::to_bytes(req.into_body()).await {
                Ok(body) => match serde_json::from_slice::<Vec<handle::Request>>(&body) {
                    Ok(requests) => {
                        let mut responses: Vec<handle::Response> = Vec::new();
                        for request in requests {
                            match command::handle_exec(request).await {
                                Ok(response) => {
                                    responses.push(response);
                                }
                                Err(e) => {
                                    let response = handle::Response::failed(e.to_string());
                                    responses.push(response);
                                }
                            }
                        }
                        responses
                    }
                    Err(e) => {
                        let response = handle::Response::failed(e.to_string());
                        vec![response]
                    }
                },
                Err(e) => {
                    let response = handle::Response::failed(e.to_string());
                    vec![response]
                }
            };
            let body = serde_json::to_vec(&responses).unwrap();
            let mut response = Response::new(Body::empty());
            let headers = response.headers_mut();
            headers.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
            headers.insert(
                "Access-Control-Allow-Methods",
                "GET, POST, PUT, DELETE".parse().unwrap(),
            );
            headers.insert(
                "Access-Control-Allow-Headers",
                "Content-Type".parse().unwrap(),
            );
            *response.body_mut() = Body::from(body);
            Ok(response)
        }
        _ => {
            // Return a 404 Not Found response for all other paths
            let response = Response::builder().status(404).body(Body::empty()).unwrap();
            Ok(response)
        }
    }
}
