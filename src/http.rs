// Imports
use std::{
    fs,
    collections::HashMap,
    io::{prelude::*, BufReader, Read, Write},
    net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener, TcpStream},
};

pub mod pool;
use crate::http::pool::ThreadPool;
pub mod parser;
use crate::http::parser::{parse_request, create_response, Response};


/// Formatea un Respone en un string para enviar
fn format_response(response: &Response) -> String {
    let status_line = match response.status_code {
        200 => "HTTP/1.1 200 OK",
        404 => "HTTP/1.1 404 NOT FOUND",
        400 => "HTTP/1.1 400 BAD REQUEST",
        _ => "HTTP/1.1 500 INTERNAL SERVER ERROR",
    };

    let headers = response
        .headers
        .iter()
        .map(|(k, v)| format!("{}: {}", k, v))
        .collect::<Vec<String>>()
        .join("\r\n");

    format!(
        "{}\r\n{}\r\n\r\n{}",
        status_line,
        headers,
        response.body.clone().unwrap_or_default()
    )
}

pub struct HttpServer;

impl HttpServer {
    pub fn new() -> HttpServer {
        HttpServer
    }

    pub fn listen(&self, port: u64, mut cb: impl FnMut() + 'static) {
        let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).unwrap();
        assert_eq!(
            listener.local_addr().unwrap(),
            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), port)),
            "[Error]: Could not open the server at the specified port"
        );
        (cb)();
        let pool = ThreadPool::new(4);

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

    // Función para manejar las conecciones
    fn handle_connection(mut stream: TcpStream) {
        let mut buf_reader = BufReader::new(&mut stream);
        let mut headers = String::new();

        // Lectura de headers (línea por línea)
        loop {
            let mut line = String::new();
            match buf_reader.read_line(&mut line) {
                Ok(0) => break, // Conexión cerrada por el cliente
                Ok(_) => {
                    if line == "\r\n" { // Cuando se llega al final de los encabezados
                        break;
                    }
                    headers.push_str(&line);
                }
                Err(e) => {
                    eprintln!("[Error]: Error reading from stream: {}", e);
                    return;
                }
            }
        }

        // Analiza el tamaño del cuerpo si Content-Length está
        let content_length = headers
            .lines()
            .find_map(|line| {
                if line.to_lowercase().starts_with("content-length:") {
                    line.split_whitespace().nth(1).and_then(|v| v.parse::<usize>().ok())
                } else {
                    None
                }
            })
            .unwrap_or(0);

        // Lectura del body en el caso de ser necesario
        let mut body = vec![0; content_length];
        if content_length > 0 {
            buf_reader.read_exact(&mut body).unwrap();
        }

        // Combina headers y body para parsear la solicitud completa
        let request_str = format!("{}{}", headers, String::from_utf8_lossy(&body));

        // Intenta parsear la solicitud
        match parse_request(&request_str) {
            Ok(request) => {
                println!("Request Parsed: {:?}", request);

                // Respuesta de ejemplo (Cambiar)
                let response = create_response(200, Some("Hello, World!".to_string()));

                // Envía la respuesta al cliente
                let response_str = format!(
                    "HTTP/1.1 {} OK\r\nContent-Length: {}\r\n\r\n{}",
                    response.status_code,
                    response.body.as_ref().map(|b| b.len()).unwrap_or(0),
                    response.body.unwrap_or_default()
                );

                stream.write_all(response_str.as_bytes()).unwrap();
                stream.flush().unwrap();
            }
            Err(e) => {
                // Si hay un error al parsear, envía un error 400
                let response = create_response(400, Some(format!("[Error]: Error parsing request: {}", e)));
                let response_str = format!(
                    "HTTP/1.1 {} Bad Request\r\nContent-Length: {}\r\n\r\n{}",
                    response.status_code,
                    response.body.as_ref().map(|b| b.len()).unwrap_or(0),
                    response.body.unwrap_or_default()
                );

                stream.write_all(response_str.as_bytes()).unwrap();
                stream.flush().unwrap();
            }
        }
    }
}