mod http;
mod app;
    
fn main() {
    let mut server = http::HttpServer::new(4);
    
    server.post("/login", app::login_controller);
    server.get("/msg", app::get_messages_controller);
    server.get("/msg/:id", app::get_message_by_id_controller);
    server.post("/msg", app::post_message_controller);
    server.put("/msg/:id", app::edit_message_controller);
    server.delete("/msg/:id", app::delete_message_by_id_controller);

    let port: u16 = 8080;
    server.listen(port, move || println!("Listening from port {}", port));
}