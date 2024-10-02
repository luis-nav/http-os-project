use crate::http::parser::{Request, Response};

// Tipo para las funciones controladoras de cada ruta
pub type Controller = fn(Request) -> Response;

// Lave para el hashmap que mapea [path, method] => controller
#[derive(PartialEq, Eq, Hash, Clone)]
pub struct RouterKey{
    pub path: String,
    pub method: String,
}