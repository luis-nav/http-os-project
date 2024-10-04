use crate::http::parser::{Body, Request, Response, create_response};
use std::sync::{Arc, RwLock, atomic::{AtomicU32, Ordering}};
use std::collections::HashMap;
extern crate lazy_static;
use app::lazy_static::lazy_static;

#[derive(Clone)]
struct Message {
    id: u32,
    content: String,
    username: String,
}

// Variable global (con Arc, RwLock y lazy_static)
lazy_static! {
    static ref MESSAGES: Arc<RwLock<HashMap<u32, Message>>> = Arc::new(RwLock::new(HashMap::new()));
}

// Variable global para el id (con AtomicU32)
lazy_static! {
    static ref NEXT_ID: AtomicU32 = AtomicU32::new(1); // Inicializamos en 1
}

// Función para leer todos los mensajes de la variable global
fn get_messages() -> Vec<Message> {
    let messages = MESSAGES.read().unwrap(); // Bloquea para lectura
    messages.iter()
        .map(|(&id, message)| Message { id, content: message.content.clone(), username: message.username.clone() })
        .collect() // Devuelve una copia de los mensajes
}

// Función para leer un mensaje por id
fn get_message(id: u32) -> Option<Message> {
    let messages = MESSAGES.read().unwrap(); // Bloquea para lectura
    messages.get(&id).cloned() // Devuelve una copia del valor
}

// Función para escribir datos en la variable global
fn add_message(content: String, username: String) -> u32 {
    let id = get_next_id(); // Obtiene un nuevo id
    let mut messages = MESSAGES.write().unwrap(); // Bloquea para escritura
    let message = Message { id, content, username }; // Crea el nuevo mensaje
    messages.insert(id, message); // Inserta el mensaje
    id
}

// Función para actualizar un mensaje por id
fn edit_existing_message(id: u32, new_content: String) -> Result<String, String> {
    let mut messages = MESSAGES.write().unwrap(); // Bloquea para escritura
    
    if let Some(message) = messages.get_mut(&id) { // Actualiza el mensaje si existe
        message.content = new_content.clone();
        Ok(format!("Message with ID {} updated", id))
    } else {
        Err(format!("Message with ID {} not found", id))
    }
}

// Función para eliminar datos de la variable global
fn delete_message(id: u32) -> Result<String, String> {
    let mut messages = MESSAGES.write().unwrap(); // Bloquea para escritura

    if messages.remove(&id).is_some() { // Elimina el mensaje si existe
        println!("Message with ID {} deleted", id);
        Ok(format!("Message with ID {} deleted", id))
    } else {
        Err("Message not found".to_string())
    }
}

// Función para obtener el siguiente id
fn get_next_id() -> u32 {
    NEXT_ID.fetch_add(1, Ordering::SeqCst) // Incrementa el id
}

// Controller para el login
pub fn login_controller(req: Request) -> Response {
    // Obtener el username desde las cookies
    let username = match req.body {
        Some(Body::Text(ref text)) => {
            text.to_string()
        }
        Some(Body::Json(ref json_value)) => {
            match json_value.get("username") {
                Some(message) => message.as_str().unwrap_or("").to_string(),
                None => return create_response(400, Some("Missing 'username' in JSON body".to_string()), None::<HashMap<String, String>>),
            }
        }
        _ => return create_response(400, Some("Invalid request body".to_string()), None::<HashMap<String, String>>),
    };
    let mut cookies:HashMap<String,String> = HashMap::new();
    cookies.insert(String::from("username"), username.clone());


    println!("User logged in: {}", username);
    create_response(200, Some(format!("Welcome, {}!", username)), Some(cookies))
}

// Controller para obtener todos los mensajes
pub fn get_messages_controller(_req: Request) -> Response {
    let messages = get_messages(); // Llamada a get_messages()
    let body = messages
        .iter()
        .map(|message| format!("{}: {} (by {})", message.id, message.content, message.username))
        .collect::<Vec<String>>()
        .join("\n");

    create_response(200, Some(body), None::<HashMap<String, String>>)
}

// Controller para obtener mensaje por id
pub fn get_message_by_id_controller(req: Request) -> Response {
    let id: u32 = req.params.get("id")
                            .and_then(|id_str| id_str.parse::<u32>().ok()) // Intenta parsear el id
                            .unwrap_or(0); // Saca el id de los params, si no hay es 0

    if let Some(message) = get_message(id) { // Llamada a get_message()
        create_response(200, Some(format!("{}: {} (by {})", message.id, message.content, message.username)), None::<HashMap<String, String>>)
    } else {
        create_response(404, Some("Message not found".to_string()), None::<HashMap<String, String>>)
    }
}

// Controller para postear un mensaje
pub fn post_message_controller(req: Request) -> Response {
    let content = match req.body {
        Some(Body::Text(ref text)) => {
            text.to_string()
        }
        Some(Body::Json(ref json_value)) => {
            match json_value.get("message") {
                Some(message) => message.as_str().unwrap_or("").to_string(),
                None => return create_response(400, Some("Missing 'message' in JSON body".to_string()), None::<HashMap<String, String>>),
            }
        }
        _ => return create_response(400, Some("Invalid request body".to_string()), None::<HashMap<String, String>>),
    };

    let username = match req.cookies.get("username") {
        Some(name) => name.clone(),
        None => return create_response(400, Some("Missing username in cookies".to_string()), None::<HashMap<String, String>>),
    };

    let id = add_message(content.clone(), username.clone()); // Llamada a add_message()

    println!("New message created with ID: {} by user: {}", id, username);
    create_response(201, Some(format!("Message created with ID: {} by user: {}", id, username)), None::<HashMap<String, String>>)
}

// Controller para editar un mensaje
pub fn edit_existing_message_controller(req: Request) -> Response {
    let id: u32 = req.params.get("id")
                            .and_then(|id_str| id_str.parse::<u32>().ok()) // Intenta parsear el id
                            .unwrap_or(0); // Saca el id de los params, si no hay es 0
    
    let new_message = match req.body {
        Some(Body::Text(ref text)) => {
            text.clone()
        }
        Some(Body::Json(ref json_value)) => {
            match json_value.get("message") {
                Some(message) => message.as_str().unwrap_or("").to_string().clone(),
                None => return create_response(400, Some("Missing 'message' in JSON body".to_string()), None::<HashMap<String, String>>),
            }
        }
        _ => return create_response(400, Some("Invalid request body".to_string()), None::<HashMap<String, String>>),
    };

    match edit_existing_message(id, new_message) { // Llamada a edit_message()
        Ok(success_msg) => create_response(200, Some(success_msg), None::<HashMap<String, String>>),
        Err(err_msg) => create_response(404, Some(err_msg), None::<HashMap<String, String>>),
    }
}

// Controller para editar un mensaje
pub fn edit_or_create_message_controller(req: Request) -> Response {
    let id: u32 = req.params.get("id")
                            .and_then(|id_str| id_str.parse::<u32>().ok()) // Intenta parsear el id
                            .unwrap_or(0); // Saca el id de los params, si no hay es 0

    let messages = MESSAGES.read().unwrap(); // Bloquea para lectura

    if id == 0 { // Error si id = 0
        return create_response(404, Some("Message not found".to_string()), None::<HashMap<String, String>>); // Respuesta 404 si id es 0
    }
    if let Some(_message) = messages.get(&id) { // Busca el mensaje si existe, solo lectura
        std::mem::drop(messages);
        return edit_existing_message_controller(req); // Llama al controlador para editar el mensaje
    } else {
        std::mem::drop(messages);
        return post_message_controller(req); // Crea un nuevo mensaje si no existe
    }
}


// Controller para eliminar un mensaje
pub fn delete_message_by_id_controller(req: Request) -> Response {
    let id: u32 = req.params.get("id")
                            .and_then(|id_str| id_str.parse::<u32>().ok()) // Intenta parsear el id
                            .unwrap_or(0); // Saca el id de los params, si no hay es 0

    match delete_message(id) { // Llamada a delete_message()
        Ok(success_msg) => create_response(200, Some(success_msg), None::<HashMap<String, String>>),
        Err(err_msg) => create_response(404, Some(err_msg), None::<HashMap<String, String>>),
    }
}