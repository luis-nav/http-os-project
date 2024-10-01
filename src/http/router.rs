use crate::http::parser::{Request, Response};

pub type Controller = fn(Request) -> Response;
// pub type Controller = Box<dyn FnMut()>;

// struct Route {
//     controller: impl FnMut() + 'static,
//     path: String
// }

// impl Route {
//     fn new(controller: impl FnMut() + 'static, path: String) -> Route {
//         Route { controller, path }
//     }
// }

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct RouterKey{
    pub path: String,
    pub method: String,
}