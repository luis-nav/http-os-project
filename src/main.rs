// use std::{
//     fs,
//     io::{prelude::*, BufReader},
//     net::{TcpListener, TcpStream},
//     thread,
//     time::Duration,
// };
mod http;

fn main() {
    println!("Hello world!");
    let app = http::HttpServer {};
    let port: u64 = 8080;
    app.listen(port, move || println!("Listening from port {}", port))
}