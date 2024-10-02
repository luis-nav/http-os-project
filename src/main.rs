mod http;
mod app;
    
fn main() {
    let mut server = http::HttpServer::new(4);
    server.get("/home", app::controller1);
    server.post("/home", app::controller2);
    let port: u16 = 8080;
    server.listen(port, move || println!("Listening from port {}", port))
}