use std::collections::HashMap;
extern crate serde_json;

/// Representa una petición HTTP.
#[derive(Debug)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: Option<Body>,
}

/// Representa una respuesta HTTP.
#[derive(Debug)]
pub struct Response {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

/// Enum para manejar diferentes tipos de bodies.
#[derive(Debug)]
pub enum Body {
    Text(String),
    Json(serde_json::Value),
}

/// Parser para convertir una petición HTTP cruda en un objeto `Request`.
pub fn parse_request(request: &str) -> Result<Request, String> {
    let mut lines = request.lines();
    let start_line = lines
        .next()
        .ok_or("Petición vacía")?
        .split_whitespace()
        .collect::<Vec<&str>>();

    if start_line.len() < 3 {
        return Err("Línea inicial inválida".to_string());
    }

    let method = start_line[0].to_string();
    let path = start_line[1].to_string();

    let mut headers = HashMap::new();
    for line in lines.by_ref() {
        if line.is_empty() {
            break; // Detecta el final de los encabezados
        }

        let parts: Vec<&str> = line.splitn(2, ": ").collect();
        if parts.len() == 2 {
            headers.insert(parts[0].to_string(), parts[1].to_string());
        }
    }

    // Determina el tamaño del cuerpo si `Content-Length` está presente
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
                    Err(_) => return Err("Error al parsear el JSON".to_string()),
                }
            }
            _ => Some(Body::Text(body)),
        }
    };

    Ok(Request {
        method,
        path,
        headers,
        body,
    })
}

/// Función para crear una respuesta básica.
pub fn create_response(status_code: u16, body: Option<String>) -> Response {
    let mut headers = HashMap::new();
    if let Some(ref body) = body {
        headers.insert("Content-Length".to_string(), body.len().to_string());
    }

    Response {
        status_code,
        headers,
        body,
    }
}