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
    app.run();
}
