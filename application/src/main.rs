#[macro_use]
extern crate log;
extern crate env_logger;
extern crate chrono;

extern crate world_gen;
extern crate graphics;
extern crate utility;

mod application;
mod application_error;
mod window;

use std::io::Write;
use env_logger::{ Builder, fmt::Formatter };
use log::Record;

use crate::application::Application;

fn main() {
    const CONFIG_PATH: &'static str = "resources/default.conf";

    init_custom_logger();

    let app = match Application::new(CONFIG_PATH) {
        Ok(app) => app,
        Err(e) => {
            error!("{}", e);
            return;
        }
    };
    match app.run() {
        Ok(_) => info!("Application exited successfully"),
        Err(e) => error!("{}", e)
    }
}

fn init_custom_logger() {
    let format = |buf: &mut Formatter , record: &Record| {
        let time = chrono::Local::now();
        writeln!(buf, "[{} {:-5}] {}", time.format("%Y-%m-%d %H:%M:%S"), record.level(), record.args()) 
    };
    Builder::from_default_env()
        .format(format)
        .init();
}
