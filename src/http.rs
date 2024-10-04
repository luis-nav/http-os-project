// Imports
use std::{
    collections::HashMap,
    io::{prelude::*, BufReader, Read, Write},
    net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener, TcpStream},
};

pub mod pool;
use crate::http::pool::ThreadPool;
pub mod parser;
use crate::http::parser::{parse_request, create_response, Response};
pub mod router;
use crate::http::router::{Controller, RouterKey};

// Función para manejar las conexiones
fn handle_connection(mut stream: TcpStream, controllers: &HashMap<RouterKey, Controller>) {
    let mut buf_reader = BufReader::new(&mut stream);
    let mut headers = String::new();

    // Lectura de headers (línea por línea)
    loop {
        let mut line = String::new();
        match buf_reader.read_line(&mut line) {
            Ok(0) => break, // Conexión cerrada por el cliente
            Ok(_) => {
                if line.trim().is_empty() { // Cuando se llega al final de los encabezados
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
    let request_str = format!("{}\r\n{}", headers, String::from_utf8_lossy(&body));

    // Intenta parsear la solicitud
    match parse_request(&request_str) {
        Ok(request) => {
            println!("Request Parsed: {:?}", request);

            let key = RouterKey { path: request.path.clone(), method: request.method.clone() };

            let controller = (controllers).get(&key);

            let response: Response;

            match controller {
                Some(func) => response = (func)(request),
                None => response = create_response(404, Some("[Error]: Route not found".to_string()), None::<HashMap<String, String>>),
            }

            // Respuesta de ejemplo (Cambiar)
            // let response = create_response(200, Some("Hello, World!".to_string()));
            let mut cookies_str = String::from("");
            if let Some(cookies) = &response.cookies {
                for (key, value) in cookies.iter() {
                    let cookie_str = format!("Set-Cookie: {}={}\r\n", key, value);
                    cookies_str.push_str(&cookie_str);
                } 
            }
            // Envía la respuesta al cliente
            let response_str = format!(
                "HTTP/1.1 {} OK\r\nContent-Length: {}\r\n{}\r\n{}",
                response.status_code,
                response.body.as_ref().map(|b| b.len()).unwrap_or(0),
                cookies_str,
                response.body.unwrap_or_default()
            );

            stream.write_all(response_str.as_bytes()).unwrap();
            stream.flush().unwrap();
        }
        Err(e) => {
            // Si hay un error al parsear, envía un error 400
            let response = create_response(400, Some(format!("[Error]: Error parsing request: {}", e)), None::<HashMap<String, String>>);
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

pub struct HttpServer {
    pool: ThreadPool,
    router: HashMap<RouterKey, Controller>
}

impl HttpServer {
    // Constructor
    pub fn new(pool_size: usize) -> HttpServer {
        HttpServer { pool: ThreadPool::new(pool_size), router: HashMap::new() }
    }

    // Add routes with controllers
    pub fn get(&mut self, path: &str, controller: Controller) {
        let key = RouterKey { path: path.to_string(), method: "GET".to_string() };
        self.router.insert(key, controller);
    }
    pub fn post(&mut self, path: &str, controller: Controller) {
        let key = RouterKey { path: path.to_string(), method: "POST".to_string() };
        self.router.insert(key, controller);
    }
    pub fn put(&mut self, path: &str, controller: Controller) {
        let key = RouterKey { path: path.to_string(), method: "PUT".to_string() };
        self.router.insert(key, controller);
    }
    pub fn patch(&mut self, path: &str, controller: Controller) {
        let key = RouterKey { path: path.to_string(), method: "PATCH".to_string() };
        self.router.insert(key, controller);
    }
    pub fn delete(&mut self, path: &str, controller: Controller) {
        let key = RouterKey { path: path.to_string(), method: "DELETE".to_string() };
        self.router.insert(key, controller);
    }

    // Start listening to ports
    pub fn listen(&self, port: u16, mut cb: impl FnMut() + 'static) {
        // Un Listener TCP para el puerto indicado
        let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).unwrap();
        // Se hace un assert para ver si el listener esta en el puerto que se indicó
        assert_eq!(
            listener.local_addr().unwrap(),
            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), port)),
            "[Error]: Could not open the server at the specified port"
        );
        // Correr el callback de que se logro abrir el puerto
        (cb)();

        // Main listener loop
        for stream in listener.incoming() {
            match stream {
                // Caso de recibir un stream al puerto
                Ok(stream) => {
                    // Duplicacion de controllers para cada thread 
                    // No se puede pasar directamente los controllers por políticas estrictas de contexto de Rust
                    let mut controllers: HashMap<RouterKey, Controller> = HashMap::new();
                    controllers.clear();
                    controllers.extend(self.router.clone().into_iter());

                    println!("[Log]: Connection Established");
                    // Ejecutar el handler de las conexiones en uno de los threads del pool
                    self.pool.execute( move || {
                        handle_connection(stream, &controllers);
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