use egui::{
    ClippedPrimitive, Context, DragValue, Label, RichText, TexturesDelta, Ui, Vec2, ViewportId,
    Window as EguiWindow,
};
use egui_wgpu::{Renderer, ScreenDescriptor, wgpu};
use egui_winit::{EventResponse, State as EguiState};
use strum::IntoEnumIterator as _;
use winit::{event::WindowEvent, window::Window};

use crate::{
    data::{
        GuiData,
        scene::{FractalGroup, PrimitiveShape},
    },
    error::GUIUnconfiguredError,
};

fn general_section(ui: &mut Ui, gui_data: &mut GuiData) {
    ui.heading(RichText::new("General settings").strong());
    ui.end_row();

    ui.label("Max iterations:")
        .on_hover_text("Maximum number of steps to take when raymarching");
    ui.add(DragValue::new(&mut gui_data.max_iterations).range(1..=1000))
        .on_hover_text("Maximum number of steps to take when raymarching");
    ui.end_row();

    ui.label("Max distance:")
        .on_hover_text("Maximum distance before we stop rendering");
    ui.add(DragValue::new(&mut gui_data.max_distance).range(10.0..=10000.0))
        .on_hover_text("Maximum distance before we stop rendering");
    ui.end_row();

    ui.label("Epsilon:")
        .on_hover_text("Accuracy of calculation");
    ui.add(
        DragValue::new(&mut gui_data.epsilon)
            .speed(0.000_001)
            .range(0.000_001..=1.0),
    )
    .on_hover_text("Accuracy of calculations");
    ui.end_row();

    ui.label("Heatmap rendering:")
        .on_hover_text("Display color via heatmap - brighter spots have higher iteration count");
    ui.checkbox(&mut gui_data.is_heatmap, "")
        .on_hover_text("Display color via heatmap - brighter spots have higher iteration count");
    ui.end_row();

    ui.label("Fractal color:");
    ui.color_edit_button_srgb(&mut gui_data.fractal_color);
    ui.end_row();

    ui.label("Background color:");
    ui.color_edit_button_srgb(&mut gui_data.background_color);
    ui.end_row();
}

fn julia_description(ui: &mut Ui, gui_data: &mut GuiData) {
    ui.label("Description:");
    ui.add(Label::new(
        "3D Julia sets are rendered by finding all quaternions that do not converge to infinity under continuous iteration of a function.\
        These are then displayed by rendering only three of the coordinate axes."
    ).wrap());
    ui.end_row();

    ui.label("Quaternion function:")
        .on_hover_text("Function used to construct the Julia set");
    ui.label(format!(
        "f(q) = q^{} + ({}, {}, {}, {})",
        match gui_data.fractal_group {
            FractalGroup::GeneralizedJuliaSet => format!("{}", gui_data.power),
            _ => "2".to_string(),
        },
        gui_data.constant.0,
        gui_data.constant.1,
        gui_data.constant.2,
        gui_data.constant.3,
    ))
    .on_hover_text("Function used to construct the Julia set");
    ui.end_row();
}

fn julia_power(ui: &mut Ui, gui_data: &mut GuiData) {
    ui.label("Power variable:")
        .on_hover_text("Power variable in quaternion function");
    ui.add(
        DragValue::new(&mut gui_data.power)
            .speed(0.01)
            .range(1.0..=10.0),
    )
    .on_hover_text("Power variable in quaternion function");
    ui.end_row();
}

fn julia_constant(ui: &mut Ui, gui_data: &mut GuiData) {
    ui.label("Constant variable:")
        .on_hover_text("Constant variable in quaternion function");
    ui.horizontal(|ui| {
        ui.style_mut().spacing.item_spacing = Vec2::new(3., 3.);
        for i in 0..4 {
            ui.add(
                DragValue::new(&mut gui_data.constant[i])
                    .speed(0.01)
                    .range(-1.0..=1.0),
            )
            .on_hover_text("Constant variable in quaternion function");
        }
    });
    ui.end_row();
}

fn fractal_group_section(ui: &mut Ui, gui_data: &mut GuiData) {
    ui.heading(RichText::new("Fractal settings").strong());
    ui.end_row();

    ui.label("Fractal group:")
        .on_hover_text("Group of fractals to display");
    egui::ComboBox::from_label("Group")
        .selected_text(format!("{}", gui_data.fractal_group))
        .show_ui(ui, |ui| {
            for group in FractalGroup::iter() {
                ui.selectable_value(&mut gui_data.fractal_group, group, format!("{group}"));
            }
        });
    ui.end_row();

    match gui_data.fractal_group {
        FractalGroup::KaleidoscopicIFS => {
            ui.label("Description:");
            ui.add(
                Label::new(
                    RichText::new("Currently can only display preset shapes. No fractals :-(.")
                        .italics(),
                )
                .wrap(),
            );
            ui.end_row();

            ui.label("Preset shapes:")
                .on_hover_text("Choose one of the preset shapes to display");
            egui::ComboBox::from_label("Shape")
                .selected_text(format!("{}", gui_data.primitive_shape))
                .show_ui(ui, |ui| {
                    for shape in PrimitiveShape::iter() {
                        ui.selectable_value(
                            &mut gui_data.primitive_shape,
                            shape,
                            format!("{shape}"),
                        );
                    }
                });
            ui.end_row();
        }
        FractalGroup::JuliaSet => {
            julia_description(ui, gui_data);
            julia_constant(ui, gui_data);
        }
        FractalGroup::GeneralizedJuliaSet => {
            julia_description(ui, gui_data);
            julia_power(ui, gui_data);
            julia_constant(ui, gui_data);
        }
    }
    ui.end_row();
}

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

            ui.end_row();

            general_section(ui, gui_data);
            ui.end_row();

            fractal_group_section(ui, gui_data);
            ui.end_row();
        });

    ui.add_space(16.);
    ui.separator();
    ui.label(RichText::new("Tip: Hover over some items for an explanation").italics());
}

pub(crate) struct GuiState {
    gui_data: GuiData,
    egui_state: EguiState,
    renderer: Renderer,
    tris: Option<Vec<ClippedPrimitive>>,
    delta: Option<TexturesDelta>,
}

impl GuiState {
    #[must_use]
    pub(crate) fn new(
        window: &Window,
        device: &wgpu::Device,
        output_color_format: wgpu::TextureFormat,
    ) -> Self {
        let gui_data = GuiData::default();
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
            egui_state,
            renderer,
            tris: None,
            delta: None,
        }
    }

    #[must_use]
    pub(crate) fn gui_data(&self) -> GuiData {
        self.gui_data
    }

    #[must_use]
    pub(crate) fn wants_pointer_input(&self) -> bool {
        self.egui_state.egui_ctx().wants_pointer_input()
    }

    #[must_use]
    pub(crate) fn wants_keyboard_input(&self) -> bool {
        self.egui_state.egui_ctx().wants_keyboard_input()
    }

    pub(crate) fn window_event(&mut self, window: &Window, event: &WindowEvent) -> EventResponse {
        self.egui_state.on_window_event(window, event)
    }

    pub(crate) fn mouse_motion(&mut self, delta: (f64, f64)) {
        self.egui_state.on_mouse_motion(delta);
    }

    pub(crate) fn update_gui(
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

        let full_output = self.egui_state.egui_ctx().run(raw_input, |_context| {
            EguiWindow::new("Settings Menu")
                .resizable(false)
                .default_open(false)
                .show(self.egui_state.egui_ctx(), |ui| {
                    update_ui(ui, &mut self.gui_data);
                });
        });

        // let full_output = self.egui_state.egui_ctx().end_pass();
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
    /// - `GUINotConfiguredError` when tried to render unconfigured GUI
    pub(crate) fn render(
        &mut self,
        render_pass: &mut wgpu::RenderPass<'static>,
        screen_descriptor: &ScreenDescriptor,
    ) -> Result<(), GUIUnconfiguredError> {
        self.renderer.render(
            render_pass,
            self.tris.as_mut().ok_or(GUIUnconfiguredError)?,
            screen_descriptor,
        );

        for id in &self.delta.as_mut().ok_or(GUIUnconfiguredError)?.free {
            self.renderer.free_texture(id);
        }

        Ok(())
    }
}
