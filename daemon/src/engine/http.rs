use std::{convert::Infallible, error::Error};

use common::config::Config;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server};
use log::{error, info};
use tokio::task::JoinHandle;

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
    let make_svc =
        make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle_connection)) });
    let server = Server::bind(&addr).serve(make_svc);
    if let Err(e) = server.await {
        error!("server error: {}", e);
    }
    Ok(())
}

async fn handle_connection(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => {
            // Handle the root path
            Ok(Response::new(Body::from("Hello, Rust HTTP server!")))
        }
        (&Method::POST, "/run") => {
            // get json body
            let body = hyper::body::to_bytes(req.into_body()).await.unwrap();

            match serde_json::from_slice(&body) {
                Ok(request) => match command::handle_exec(request).await {
                    Ok(response) => {
                        let body = serde_json::to_vec(&response).unwrap();
                        Ok(Response::new(Body::from(body)))
                    }
                    Err(e) => {
                        let body = format!("command exec error: {}", e);
                        let response = Response::builder()
                            .status(500)
                            .body(Body::from(body))
                            .unwrap();
                        Ok(response)
                    }
                },
                Err(e) => {
                    let body = format!("json parse error: {}", e);
                    let response = Response::builder()
                        .status(400)
                        .body(Body::from(body))
                        .unwrap();
                    Ok(response)
                }
            }
        }
        _ => {
            // Return a 404 Not Found response for all other paths
            let response = Response::builder().status(404).body(Body::empty()).unwrap();
            Ok(response)
        }
    }
}
