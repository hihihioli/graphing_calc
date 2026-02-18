use eframe::egui;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AAMode {
    None,
    FXAA,
    TAA,
}

impl AAMode {
    pub fn as_str(&self) -> &str {
        match self {
            AAMode::None => "None",
            AAMode::FXAA => "FXAA",
            AAMode::TAA => "TAA",
        }
    }
}

#[derive(Debug, Clone)]
pub struct GraphParams {
    pub center_x: f64,
    pub center_y: f64,
    pub zoom: f64,
    pub aa_mode: AAMode,
    pub bloom_enabled: bool,
    pub color_static: bool,
}

impl Default for GraphParams {
    fn default() -> Self {
        Self {
            center_x: 0.0,
            center_y: 0.0,
            zoom: 20.0,
            aa_mode: AAMode::TAA,
            bloom_enabled: true,
            color_static: false,
        }
    }
}

struct ControlApp {
    params: Arc<Mutex<GraphParams>>,
    center_x_str: String,
    center_y_str: String,
    zoom_str: String,
    last_synced_x: f64,
    last_synced_y: f64,
    last_synced_zoom: f64,
}

impl ControlApp {
    fn new(params: Arc<Mutex<GraphParams>>) -> Self {
        Self {
            params,
            center_x_str: String::from("0.0"),
            center_y_str: String::from("0.0"),
            zoom_str: String::from("20.0"),
            last_synced_x: 0.0,
            last_synced_y: 0.0,
            last_synced_zoom: 20.0,
        }
    }
}

impl eframe::App for ControlApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Sync text fields from params if they changed externally (e.g., mouse dragging)
        if let Ok(p) = self.params.lock() {
            if (p.center_x - self.last_synced_x).abs() > 1e-6 {
                self.center_x_str = format!("{:.4}", p.center_x);
                self.last_synced_x = p.center_x;
            }
            if (p.center_y - self.last_synced_y).abs() > 1e-6 {
                self.center_y_str = format!("{:.4}", p.center_y);
                self.last_synced_y = p.center_y;
            }
            if (p.zoom - self.last_synced_zoom).abs() > 1e-6 {
                self.zoom_str = format!("{:.4}", p.zoom);
                self.last_synced_zoom = p.zoom;
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Graphing Calculator Controls");
            ui.add_space(10.0);

            ui.group(|ui| {
                ui.label("View Settings");
                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    ui.label("X Center:");
                    if ui.text_edit_singleline(&mut self.center_x_str).changed() {
                        if let Ok(val) = self.center_x_str.parse::<f64>() {
                            if let Ok(mut p) = self.params.lock() {
                                p.center_x = val;
                                self.last_synced_x = val;
                            }
                        }
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Y Center:");
                    if ui.text_edit_singleline(&mut self.center_y_str).changed() {
                        if let Ok(val) = self.center_y_str.parse::<f64>() {
                            if let Ok(mut p) = self.params.lock() {
                                p.center_y = val;
                                self.last_synced_y = val;
                            }
                        }
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Zoom:");
                    if ui.text_edit_singleline(&mut self.zoom_str).changed() {
                        if let Ok(val) = self.zoom_str.parse::<f64>() {
                            if val > 0.0 {
                                if let Ok(mut p) = self.params.lock() {
                                    p.zoom = val;
                                    self.last_synced_zoom = val;
                                }
                            }
                        }
                    }
                });
            });

            ui.add_space(10.0);

            if ui.button("Reset View").clicked() {
                self.center_x_str = String::from("0.0");
                self.center_y_str = String::from("0.0");
                self.zoom_str = String::from("20.0");
                self.last_synced_x = 0.0;
                self.last_synced_y = 0.0;
                self.last_synced_zoom = 20.0;
                if let Ok(mut p) = self.params.lock() {
                    p.center_x = 0.0;
                    p.center_y = 0.0;
                    p.zoom = 20.0;
                }
            }

            ui.add_space(15.0);

            // Rendering settings
            ui.group(|ui| {
                ui.label("Rendering Settings");
                ui.add_space(5.0);

                if let Ok(mut p) = self.params.lock() {
                    // AA Mode dropdown
                    ui.horizontal(|ui| {
                        ui.label("Anti-Aliasing:");
                        egui::ComboBox::new("aa_mode", "")
                            .selected_text(p.aa_mode.as_str())
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut p.aa_mode, AAMode::None, "None");
                                ui.selectable_value(&mut p.aa_mode, AAMode::FXAA, "FXAA");
                                ui.selectable_value(&mut p.aa_mode, AAMode::TAA, "TAA");
                            });
                    });
                    
                    ui.checkbox(&mut p.bloom_enabled, "Bloom Effect");
                    ui.checkbox(&mut p.color_static, "Static Colors");
                }
            });

            ui.add_space(10.0);

            // Display current values
            if let Ok(p) = self.params.lock() {
                ui.separator();
                ui.label(format!("Current X: {:.4}", p.center_x));
                ui.label(format!("Current Y: {:.4}", p.center_y));
                ui.label(format!("Current Zoom: {:.4}", p.zoom));
            }

            ui.add_space(10.0);
            
            ui.label("Tip: You can also drag the graph window with your mouse");
        });

        ctx.request_repaint();
    }
}

pub fn run_ui_window(params: Arc<Mutex<GraphParams>>) {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Graphing Calculator Controls")
            .with_inner_size([400.0, 450.0])
            .with_resizable(true),
        event_loop_builder: Some(Box::new(|_builder| {
            #[cfg(target_os = "windows")]
            {
                use winit::platform::windows::EventLoopBuilderExtWindows;
                _builder.with_any_thread(true);
            }
            #[cfg(target_os = "linux")]
            {
                use winit::platform::x11::EventLoopBuilderExtX11;
                _builder.with_any_thread(true);
            }
        })),
        ..Default::default()
    };

    let _ = eframe::run_native(
        "Graphing Calculator Controls",
        native_options,
        Box::new(|_cc| Ok(Box::new(ControlApp::new(params)))),
    );
}
