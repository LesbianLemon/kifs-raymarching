use kifs_raymarching::render;

fn run() {
    env_logger::init();

    let options = render::RenderStateOptions::default();

    match render::run(options) {
        Ok(()) => log::info!("Event loop finished. Exiting program..."),
        Err(err) => log::error!("{err}"),
    }
}

fn main() {
    run();
}
