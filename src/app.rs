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
fn edit_message(id: u32, new_content: String) -> Result<String, String> {
    let mut messages = MESSAGES.write().unwrap(); // Bloquea para escritura
    
    if let Some(message) = messages.get_mut(&id) { // Actualiza el mensaje si existe
        message.content = new_content.clone();
        Ok(format!("Message with ID {} updated", id))
    } else {
        Err(format!("Message with ID {} not found", id))
    }
}

// Función para eliminar datos de la variable global
pub fn delete_message(id: u32) -> Result<String, String> {
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
    let username = match req.body {
        Some(Body::Text(ref text)) => text.clone(),
        _ => return create_response(400, Some("Invalid request body".to_string())),
    };

    println!("User logged in: {}", username);
    create_response(200, Some(format!("Welcome, {}!", username)))
}

// Controller para obtener todos los mensajes
pub fn get_messages_controller(_req: Request) -> Response {
    let messages = get_messages(); // Llamada a get_messages()
    let body = messages
        .iter()
        .map(|message| format!("{}: {} (by {})", message.id, message.content, message.username))
        .collect::<Vec<String>>()
        .join("\n");

    create_response(200, Some(body))
}

// Controller para obtener mensaje por id
pub fn get_message_by_id_controller(req: Request) -> Response {
    let id: u32 = req.path.trim_start_matches("/msg/")
                        .parse()
                        .unwrap_or(0); // Parsing del id

    if let Some(message) = get_message(id) { // Llamada a get_message()
        create_response(200, Some(format!("{}: {} (by {})", message.id, message.content, message.username)))
    } else {
        create_response(404, Some("Message not found".to_string()))
    }
}

// Controller para postear un mensaje
pub fn post_message_controller(req: Request) -> Response {
    let (content, username) = match req.body {
        Some(Body::Text(ref text)) => {
            let parts: Vec<&str> = text.splitn(2, '|').collect();
            if parts.len() == 2 {
                (parts[1].to_string(), parts[0].to_string())
            } else {
                return create_response(400, Some("Invalid request body".to_string()));
            }
        }
        _ => return create_response(400, Some("Invalid request body".to_string())),
    };

    let id = add_message(content.clone(), username.clone()); // Llamada a add_message()

    println!("New message created with ID: {} by user: {}", id, username);
    create_response(201, Some(format!("Message created with ID: {} by user: {}", id, username)))
}

// Controller para editar un mensaje
pub fn edit_message_controller(req: Request) -> Response {
    let id: u32 = req.path.trim_start_matches("/msg/")
                        .parse()
                        .unwrap_or(0); // Parsing del id
    
    let new_message = match req.body {
        Some(Body::Text(ref text)) => text.clone(),
        _ => return create_response(400, Some("Invalid request body".to_string())),
    };

    match edit_message(id, new_message) { // Llamada a edit_message()
        Ok(success_msg) => create_response(200, Some(success_msg)),
        Err(err_msg) => create_response(404, Some(err_msg)),
    }
}

// Controller para eliminar un mensaje
pub fn delete_message_by_id_controller(req: Request) -> Response {
    let id: u32 = req.path.trim_start_matches("/msg/")
                        .parse()
                        .unwrap_or(0);  // Parsing del id

    match delete_message(id) { // Llamada a delete_message()
        Ok(success_msg) => create_response(200, Some(success_msg)),
        Err(err_msg) => create_response(404, Some(err_msg)),
    }
}