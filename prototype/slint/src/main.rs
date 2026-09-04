mod models;
mod storage;
mod core;
mod services;
mod platform;
mod app;

fn main() {
    let app = app::Application::new();
    app.run();
}