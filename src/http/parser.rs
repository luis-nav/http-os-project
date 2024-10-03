// Imports
use std::collections::HashMap;
extern crate serde_json;

// Struct de Request
#[derive(Debug)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: Option<Body>,
    pub params: HashMap<String, String>,
    pub cookies: HashMap<String, String>,
}

// Struct de Response
#[derive(Debug)]
pub struct Response {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
    pub cookies: Option<HashMap<String, String>>,
}

/// Enum para manejar tipos del body
#[derive(Debug)]
pub enum Body {
    Text(String),
    Json(serde_json::Value),
}

// Parser: convertirte un request HTTP en un objeto Request
pub fn parse_request(request: &str) -> Result<Request, String> {
    let mut lines = request.lines();
    let start_line = lines
        .next()
        .ok_or("[Error]: Empty petition")?
        .split_whitespace()
        .collect::<Vec<&str>>();

    if start_line.len() < 3 {
        return Err("[Error]: Invalid initial line".to_string());
    }

    let method = start_line[0].to_string();
    let mut path = start_line[1].to_string();
    
    let mut headers = HashMap::new();
    for line in lines.by_ref() {
        if line.is_empty() {
            break;
        }
        
        let parts: Vec<&str> = line.splitn(2, ": ").collect();
        if parts.len() == 2 {
            headers.insert(parts[0].to_string(), parts[1].to_string());
        }
    }
    
    // Determina el tamaño del cuerpo si Content-Length está
    let content_length = headers
    .get("Content-Length")
    .and_then(|v| v.parse::<usize>().ok())
    .unwrap_or(0);

    let mut body = lines.collect::<Vec<&str>>().join("\n");
    if body.len() > content_length {
        body.truncate(content_length); // Trunca el cuerpo según Content-Length
    }

    let body = if body.trim().is_empty() {
        None
    } else {
        match headers.get("Content-Type") {
            Some(content_type) if content_type.contains("application/json") => {
                match serde_json::from_str(&body) {
                    Ok(json) => Some(Body::Json(json)),
                    Err(_) => return Err("[Error]: Error parsing JSON".to_string()),
                }
            }
            _ => Some(Body::Text(body)),
        }
    };
    // Filtrar el nombre del host del path
    if let Some(host) = headers.get("Host") {
        if let Some(found_idx) = path.find(host) {
            path = path.split_off(found_idx+host.len());
        }
    }
    
    // Filtrar y almacenar request params
    let mut params: HashMap<String, String> = HashMap::new();
    if let Some(found_idx) = path.find('?') {
        let params_str = path.split_off(found_idx + 1);
        if !params_str.is_empty() {
            for param_str in params_str.split('&') {
                if let Some((key, value)) = param_str.split_once('=') {
                    params.insert(key.to_string(), value.to_string());
                }
            }
        }
    }

    // Identificar cookies
    let mut cookies: HashMap<String, String> = HashMap::new();
    if let Some(cookies_str) = headers.get("Cookie") {
        for cookie_str in cookies_str.split(';') {
            let cookie_str = cookie_str.trim();
            if let Some((key, value)) = cookie_str.split_once('=') {
                cookies.insert(key.to_string(), value.to_string());
            }
        }
    }

    Ok(Request {
        method,
        path,
        headers,
        body,
        params,
        cookies,
    })
}

// Función para crear un response
pub fn create_response(status_code: u16, body: Option<String>) -> Response {
    let mut headers = HashMap::new();
    if let Some(ref body) = body {
        headers.insert("Content-Length".to_string(), body.len().to_string());
    }

    Response {
        status_code,
        headers,
        body,
        cookies: None,
    }
}

/// Formatea un Respone en un string para enviar
fn format_response(response: &Response) -> String {
    let status_line = match response.status_code {
        200 => "HTTP/1.1 200 OK",
        404 => "HTTP/1.1 404 NOT FOUND",
        400 => "HTTP/1.1 400 BAD REQUEST",
        _ => "HTTP/1.1 500 INTERNAL SERVER ERROR",
    };

    let mut headers = response
        .headers
        .iter()
        .map(|(k, v)| format!("{}: {}", k, v))
        .collect::<Vec<String>>()
        .join("\r\n");
    
    // Add cookies
    if let Some(cookies) = &response.cookies {
        for (key, value) in cookies.iter() {
            let cookie_str = format!("Set-Cookie: {}={}\r\n", key, value);
            headers.push_str(&cookie_str);
        } 
    }
    

    format!(
        "{}\r\n{}\r\n\r\n{}",
        status_line,
        headers,
        response.body.clone().unwrap_or_default()
    )
}