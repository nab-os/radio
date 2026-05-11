use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use crate::middleware::Middleware;

mod args;
mod audio;
mod data;
mod library;
mod middleware;
mod net;

fn main() {
    let args = args::parse();

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    let mut middleware = Middleware::new(args.music_path, args.bind);
    middleware.start();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    while running.load(Ordering::SeqCst) {}

    middleware.stop();
}
