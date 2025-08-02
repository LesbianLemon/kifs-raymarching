use egui_wgpu::wgpu;
use kifs_raymarching::application::Application;
use kifs_raymarching::render::RenderStateOptions;

fn main() {
    env_logger::init();

    let state_options = RenderStateOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        ..RenderStateOptions::default()
    };
    let mut app = Application::new(state_options);

    match app.run() {
        Ok(()) => log::info!("Application exited successfully without any errors"),
        Err(error) => log::error!("Application came into an unrecoverable error: {error}"),
    }
}
