use egui_wgpu::{Renderer, ScreenDescriptor, wgpu};

use egui::{ClippedPrimitive, Context, TexturesDelta, ViewportId};
use egui_winit::{EventResponse, State};
use winit::{event::WindowEvent, window::Window};

pub struct GuiState {
    state: State,
    renderer: Renderer,
    tris: Option<Vec<ClippedPrimitive>>,
    delta: Option<TexturesDelta>,
}

impl GuiState {
    pub fn new(
        window: &Window,
        device: &wgpu::Device,
        output_color_format: wgpu::TextureFormat,
    ) -> Self {
        let state = State::new(
            Context::default(),
            ViewportId::ROOT,
            window,
            Some(window.scale_factor() as f32),
            None,
            Some(1024),
        );

        let renderer = Renderer::new(device, output_color_format, None, 1, true);

        Self {
            state,
            renderer,
            tris: None,
            delta: None,
        }
    }

    pub fn wants_pointer_input(&self) -> bool {
        self.state.egui_ctx().wants_pointer_input()
    }

    pub fn wants_keyboard_input(&self) -> bool {
        self.state.egui_ctx().wants_keyboard_input()
    }

    pub fn window_event(&mut self, window: &Window, event: &WindowEvent) -> EventResponse {
        self.state.on_window_event(window, event)
    }

    pub fn mouse_motion(&mut self, delta: (f64, f64)) {
        self.state.on_mouse_motion(delta)
    }

    pub fn prerender(
        &mut self,
        window: &Window,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        screen_descriptor: &ScreenDescriptor,
    ) {
        let pixels_per_point = screen_descriptor.pixels_per_point;
        self.state.egui_ctx().set_pixels_per_point(pixels_per_point);

        let raw_input = self.state.take_egui_input(window);
        self.state.egui_ctx().begin_pass(raw_input);

        egui::Window::new("winit + egui + wgpu says hello!")
            .resizable(true)
            .vscroll(true)
            .default_open(false)
            .show(self.state.egui_ctx(), |ui| {
                ui.label("Label!");

                if ui.button("Button!").clicked() {
                    println!("boom!")
                }

                ui.separator();
                ui.horizontal(|ui| {
                    ui.label(format!(
                        "Pixels per point: {}",
                        self.state.egui_ctx().pixels_per_point()
                    ));
                });
            });

        let full_output = self.state.egui_ctx().end_pass();
        self.state
            .handle_platform_output(window, full_output.platform_output);

        let tris = self
            .state
            .egui_ctx()
            .tessellate(full_output.shapes, self.state.egui_ctx().pixels_per_point());

        for (id, image_delta) in &full_output.textures_delta.set {
            self.renderer
                .update_texture(device, queue, *id, image_delta);
        }

        self.renderer
            .update_buffers(device, queue, encoder, &tris, screen_descriptor);

        self.tris = Some(tris);
        self.delta = Some(full_output.textures_delta);
    }

    pub fn render(&mut self, render_pass: wgpu::RenderPass, screen_descriptor: &ScreenDescriptor) {
        self.renderer.render(
            &mut render_pass.forget_lifetime(),
            self.tris.as_mut().unwrap(),
            screen_descriptor,
        );

        for x in &self.delta.as_mut().unwrap().free {
            self.renderer.free_texture(x)
        }
    }
}
