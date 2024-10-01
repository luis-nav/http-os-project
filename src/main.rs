// use std::{
//     fs,
//     io::{prelude::*, BufReader},
//     net::{TcpListener, TcpStream},
//     thread,
//     time::Duration,
// };
use std::collections::HashMap;
mod http;

fn controller1 (req: http::parser::Request) -> http::parser::Response {
    println!("{}", req.method);
    http::parser::Response {
        status_code: 200, 
        headers: req.headers, 
        body: Some("Hello, World!".to_string())
    }
}

fn controller2 (req: http::parser::Request) -> http::parser::Response {
    println!("Esto es controller 2{}", req.method);
    http::parser::Response {
        status_code: 200, 
        headers: req.headers, 
        body: Some("Hello, World!".to_string())
    }
}
    
fn main() {
    let mut router = HashMap::<http::router::RouterKey, http::router::Controller>::new();
    let key1 =  http::router::RouterKey { path: "/home".to_string(), method: "GET".to_string() };
    router.insert(key1, controller1);
    let key2 =  http::router::RouterKey { path: "/home".to_string(), method: "POST".to_string() };
    router.insert(key2, controller2);
    let app = http::HttpServer::new(4, router);
    let port: u16 = 8080;
    app.listen(port, move || println!("Listening from port {}", port))
}