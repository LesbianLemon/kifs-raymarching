use egui::{ClippedPrimitive, Context, TexturesDelta, Ui, ViewportId};
use egui_wgpu::{Renderer, ScreenDescriptor, wgpu};
use egui_winit::{EventResponse, State as EguiState};
use winit::{event::WindowEvent, window::Window};

use crate::{
    data::{
        GuiData,
        scene::PrimitiveShape,
        uniform::{UniformBuffer, UniformBufferDescriptor, UniformBufferInit},
    },
    util::error::GUINotConfiguredError,
};

struct GuiGenerator;

impl GuiGenerator {
    fn update_ui(ui: &mut Ui, gui_data: &mut GuiData) {
        egui::Grid::new("main_grid")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                ui.label("GUI Theme");
                let mut theme_preference = ui.ctx().options(|opt| opt.theme_preference);
                theme_preference.radio_buttons(ui);
                ui.ctx().set_theme(theme_preference);
                ui.end_row();

                ui.label("Label!");
                ui.end_row();

                ui.label("Butpn");
                if ui.button("Button!").clicked() {
                    println!("boom!");
                }
                ui.end_row();

                ui.label("Checkboxxxxxxxx");
                ui.checkbox(&mut true, "Checkbox");
                ui.end_row();

                ui.label("Slajdr");
                ui.add(egui::Slider::new(&mut 180.0, 0.0..=360.0).suffix("°"));
                ui.end_row();

                ui.label("Primitive shape");
                egui::ComboBox::from_label("Shape")
                    .selected_text(format!("{:?}", gui_data.primitive_shape))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut gui_data.primitive_shape,
                            PrimitiveShape::Sphere,
                            format!("{:?}", PrimitiveShape::Sphere),
                        );
                        ui.selectable_value(
                            &mut gui_data.primitive_shape,
                            PrimitiveShape::Cylinder,
                            format!("{:?}", PrimitiveShape::Cylinder),
                        );
                        ui.selectable_value(
                            &mut gui_data.primitive_shape,
                            PrimitiveShape::Box,
                            format!("{:?}", PrimitiveShape::Box),
                        );
                        ui.selectable_value(
                            &mut gui_data.primitive_shape,
                            PrimitiveShape::Torus,
                            format!("{:?}", PrimitiveShape::Torus),
                        );
                        ui.selectable_value(
                            &mut gui_data.primitive_shape,
                            PrimitiveShape::SierpinskiTetrahedron,
                            format!("{:?}", PrimitiveShape::SierpinskiTetrahedron),
                        );
                        ui.selectable_value(
                            &mut gui_data.primitive_shape,
                            PrimitiveShape::Bunny,
                            format!("{:?}", PrimitiveShape::Bunny),
                        );
                    });
                ui.end_row();

                ui.label("Fractal color");
                ui.color_edit_button_srgba(&mut gui_data.fractal_color);
                ui.end_row();

                ui.label("Background color");
                ui.color_edit_button_srgba(&mut gui_data.background_color);
                ui.end_row();
            });
    }
}

pub struct GuiState {
    gui_data: GuiData,
    gui_uniform_buffer: UniformBuffer,
    egui_state: EguiState,
    renderer: Renderer,
    tris: Option<Vec<ClippedPrimitive>>,
    delta: Option<TexturesDelta>,
}

impl GuiState {
    #[must_use]
    pub fn new(
        window: &Window,
        device: &wgpu::Device,
        output_color_format: wgpu::TextureFormat,
    ) -> Self {
        let gui_data = GuiData::default();
        let gui_uniform_buffer = device.create_uniform_buffer(&UniformBufferDescriptor {
            label: Some("gui_uniform_buffer"),
            data: gui_data,
        });

        let egui_state = EguiState::new(
            Context::default(),
            ViewportId::ROOT,
            window,
            #[allow(clippy::cast_possible_truncation)]
            Some(window.scale_factor() as f32),
            None,
            Some(1024),
        );

        let renderer = Renderer::new(device, output_color_format, None, 1, true);

        Self {
            gui_data,
            gui_uniform_buffer,
            egui_state,
            renderer,
            tris: None,
            delta: None,
        }
    }

    #[must_use]
    pub fn gui_data(&self) -> GuiData {
        self.gui_data
    }

    #[must_use]
    pub fn gui_uniform_buffer(&self) -> &UniformBuffer {
        &self.gui_uniform_buffer
    }

    #[must_use]
    pub fn wants_pointer_input(&self) -> bool {
        self.egui_state.egui_ctx().wants_pointer_input()
    }

    #[must_use]
    pub fn wants_keyboard_input(&self) -> bool {
        self.egui_state.egui_ctx().wants_keyboard_input()
    }

    pub fn window_event(&mut self, window: &Window, event: &WindowEvent) -> EventResponse {
        self.egui_state.on_window_event(window, event)
    }

    pub fn mouse_motion(&mut self, delta: (f64, f64)) {
        self.egui_state.on_mouse_motion(delta);
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
        self.egui_state
            .egui_ctx()
            .set_pixels_per_point(pixels_per_point);

        let raw_input = self.egui_state.take_egui_input(window);
        self.egui_state.egui_ctx().begin_pass(raw_input);

        egui::Window::new("Settings Menu")
            .resizable(true)
            .vscroll(true)
            .default_open(false)
            .show(self.egui_state.egui_ctx(), |ui| {
                GuiGenerator::update_ui(ui, &mut self.gui_data);
            });
        self.gui_uniform_buffer.update_buffer(queue, self.gui_data);

        let full_output = self.egui_state.egui_ctx().end_pass();
        self.egui_state
            .handle_platform_output(window, full_output.platform_output);

        let tris = self.egui_state.egui_ctx().tessellate(
            full_output.shapes,
            self.egui_state.egui_ctx().pixels_per_point(),
        );

        for (id, image_delta) in &full_output.textures_delta.set {
            self.renderer
                .update_texture(device, queue, *id, image_delta);
        }

        self.renderer
            .update_buffers(device, queue, encoder, &tris, screen_descriptor);

        self.tris = Some(tris);
        self.delta = Some(full_output.textures_delta);
    }

    /// ## Errors
    /// - `GUINotConfiguredError` when tried to render prior to first configuring the GUI
    pub fn render(
        &mut self,
        render_pass: wgpu::RenderPass,
        screen_descriptor: &ScreenDescriptor,
    ) -> Result<(), GUINotConfiguredError> {
        self.renderer.render(
            &mut render_pass.forget_lifetime(),
            self.tris.as_mut().ok_or(GUINotConfiguredError)?,
            screen_descriptor,
        );

        for id in &self.delta.as_mut().ok_or(GUINotConfiguredError)?.free {
            self.renderer.free_texture(id);
        }

        Ok(())
    }
}
