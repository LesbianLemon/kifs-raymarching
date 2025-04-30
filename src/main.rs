use kifs_raymarching::application::Application;
use kifs_raymarching::render::RenderStateOptions;

fn main() {
    env_logger::init();

    let state_options = RenderStateOptions::default();
    let mut app = Application::new(state_options);

    match app.run() {
        Ok(()) => log::info!("Event loop finished. Exiting program..."),
        Err(err) => log::error!("{err}"),
    }
}
