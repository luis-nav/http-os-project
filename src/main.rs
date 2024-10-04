mod http;
mod app;
    
fn main() {
    let mut server = http::HttpServer::new(10);
    
    server.post("/login", app::login_controller);
    server.get("/msg", app::get_messages_controller);
    server.get("/msg?", app::get_message_by_id_controller);
    server.post("/msg", app::post_message_controller);
    server.patch("/msg?", app::edit_existing_message_controller);
    server.put("/msg?", app::edit_or_create_message_controller);
    server.delete("/msg?", app::delete_message_by_id_controller);

    let port: u16 = 8080;
    server.listen(port, move || println!("Listening from port {}", port));
}