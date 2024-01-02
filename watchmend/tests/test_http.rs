#[cfg(test)]
mod tests {
    use hyper::{Body, Method, Request};
    use watchmend::common::{config::Config, handle, task::Task};

    #[tokio::test]
    async fn test_request() {
        let config: Config = Config::init(None).unwrap();
        let host = config.http.host;
        let port = config.http.port;

        let request = handle::Request {
            command: handle::Command::Run(Task::default()),
        };
        let buf = serde_json::to_vec(&[request]).unwrap();

        let uri = format!("http://{}:{}/api", host, port);
        let request = Request::builder()
            .method(Method::POST)
            .uri(uri)
            .body(Body::from(buf))
            .unwrap();

        let client = hyper::Client::new();
        let response = client.request(request).await.unwrap();

        let data = hyper::body::to_bytes(response.into_body()).await.unwrap();

        let res: Vec<handle::Response> = serde_json::from_slice(&data).unwrap();
        println!("{:#?}", res);
    }
}
