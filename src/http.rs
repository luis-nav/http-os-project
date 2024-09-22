use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener, TcpStream},
};

pub mod pool;
use crate::http::pool::ThreadPool;

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();
    
    let (status_line, filename) = if request_line == "GET / HTTP/1.1" {
        ("HTTP/1.1 200 OK", "hello.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "404.html")
    };

    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();

    let response =
        format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
    // println!("Request: {http_request:#?}");
}

// Aplication struct
pub struct HttpServer;
//{
    // pub pool: ThreadPool;
//}

impl HttpServer{
    // Constructor HttpServer
    pub fn new() -> HttpServer {
        HttpServer
    }

    // Opens port and listens for new streams of data.
    pub fn listen(&self, port: u64, mut cb: impl FnMut() + 'static) {
        // Try to open port
        let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).unwrap();
        assert_eq!(listener.local_addr().unwrap(),
            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 8080)),
            "[Error]: Could not open the server at the specified port"
        );
        // Run callback once the port has been opened correctly
        (cb)();
        // Creates the thread pool
        let pool = ThreadPool::new(4);
        // Listen for new streams and assign threads to streams
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    println!("Connection Established");
                    pool.execute(|| {
                        handle_connection(stream);
                    });
                }
                Err(e) => {
                    println!("[Error]: Failed to establish a connection: {}", e);
                }
            }
        }

        println!("Shutting down.");
    }
}