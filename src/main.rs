use kifs_raymarching::application;
use kifs_raymarching::render;

fn main() {
    env_logger::init();

    let options = render::RenderStateOptions::default();

    match application::run(options) {
        Ok(()) => log::info!("Event loop finished. Exiting program..."),
        Err(err) => log::error!("{err}"),
    }
}
