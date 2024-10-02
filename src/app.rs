use crate::http::parser::{Request, Response};

pub fn controller1 (req: Request) -> Response {
    println!("{}", req.method);
    Response {
        status_code: 200, 
        headers: req.headers, 
        body: Some("Hello, World!".to_string())
    }
}

pub fn controller2 (req: Request) -> Response {
    println!("Esto es controller 2{}", req.method);
    Response {
        status_code: 200, 
        headers: req.headers, 
        body: Some("Hello, World!".to_string())
    }
}