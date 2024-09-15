use std::{
    io::{prelude::*, BufReader},
    net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener, TcpStream},
};

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    println!("Request: {http_request:#?}");
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
        // Listen for new streams and assign threads to streams
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    println!("Connection Established");
                    handle_connection(stream);
                }
                Err(e) => {
                    println!("[Error]: Failed to establish a connection: {}", e);
                }
            }
        }

    }
}