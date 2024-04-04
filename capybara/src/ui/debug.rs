use crate::egui::Align;
use crate::egui::Color32;
use crate::egui::Context;
use crate::egui::Frame;
use crate::egui::Margin;
use crate::egui::RichText;
use crate::egui::Rounding;
use crate::egui::ScrollArea;
use crate::egui::Stroke;
use crate::egui::TextEdit;
use crate::egui::TextStyle;
use crate::egui::Vec2;
use crate::egui::Vec2b;
use crate::egui::Window;
use crate::egui_plot::Line;
use crate::egui_plot::LineStyle;
use crate::egui_plot::Plot;
use crate::egui_plot::PlotPoint;
use crate::egui_plot::PlotPoints;
use crate::utils::debug::DebugCollector;
use crate::utils::debug::DebugConsole;
use crate::utils::profiler::Profiler;

#[derive(Default)]
pub struct DebugWindow {
    pub plot_definitions: Vec<ProfilerPlotDefinition>,
}

pub struct ProfilerPlotDefinition {
    pub name: String,
    pub label: String,
    pub color: Color32,
}

impl DebugWindow {
    pub fn show(&self, context: &Context, console: &mut DebugConsole, profiler: &Profiler, collector: &mut DebugCollector) {
        let data = collector.get_data();
        let mut plot_data = Vec::default();

        Window::new("Debug")
            .frame(
                Frame::none()
                    .inner_margin(Margin::symmetric(10.0, 10.0))
                    .stroke(Stroke::new(1.0, Color32::from_rgba_unmultiplied(0, 0, 0, 240)))
                    .fill(Color32::from_rgba_unmultiplied(40, 40, 40, 240))
                    .rounding(Rounding::same(5.0)),
            )
            .resizable(false)
            .collapsible(false)
            .title_bar(false)
            .constrain(false)
            .default_width(700.0)
            .show(context, |ui| {
                ui.style_mut().visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, Color32::from_rgb(255, 255, 255));
                ui.style_mut().override_text_style = Some(TextStyle::Name("Debug".into()));

                ui.label(format!("{}, {}x{}", data.hardware_info, data.resolution.x, data.resolution.y));
                ui.add_space(15.0);
                ui.columns(4, |ui| {
                    ui[0].vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(format!("FPS average: {}", data.fps_average));
                        });
                        ui.horizontal(|ui| {
                            ui.style_mut().spacing.item_spacing = egui::Vec2::new(4.0, 0.0);
                            ui.label("Delta current:");
                            ui.label(RichText::new(format!("{:.1} ms", data.delta_current * 1000.0)).color(Color32::GREEN));
                        });
                        ui.horizontal(|ui| {
                            ui.label(format!("Delta average: {:.1} ms", data.delta_average * 1000.0));
                        });
                        ui.horizontal(|ui| {
                            ui.label(format!("Delta deviation: {:.1} ms", data.delta_deviation * 1000.0));
                        });
                    });

                    ui[1].horizontal(|ui| {
                        let plot = Plot::new("DeltaPlot")
                            .height(100.0)
                            .auto_bounds_x()
                            .auto_bounds_y()
                            .set_margin_fraction(Vec2::new(0.0, 0.1))
                            .include_y(0.0)
                            .include_y(10.0)
                            .allow_zoom(false)
                            .allow_drag(false)
                            .allow_scroll(false)
                            .allow_double_click_reset(false)
                            .allow_boxed_zoom(false)
                            .show_x(false)
                            .show_y(false)
                            .y_axis_width(1)
                            .x_axis_formatter(|_, _, _| "".to_string())
                            .show_grid(Vec2b::new(false, true));
                        let plot_data = (0..collector.delta_history_capacity)
                            .map(|i| PlotPoint::new(i as f32, *collector.delta_history.get(i).unwrap_or(&0.0) * 1000.0))
                            .collect::<Vec<PlotPoint>>();

                        plot.show(ui, |plot_ui| {
                            plot_ui.line(Line::new(PlotPoints::Owned(plot_data)).color(Color32::GREEN).style(LineStyle::Solid));
                        });
                    });

                    ui[2].horizontal(|ui| {
                        ui.add_space(15.0);
                        ui.vertical(|ui| {
                            for definition in &self.plot_definitions {
                                if let Some(data) = profiler.data.get(&definition.name) {
                                    let average = data.history.iter().sum::<f32>() / data.history.len() as f32;
                                    ui.horizontal(|ui| {
                                        ui.style_mut().spacing.item_spacing = egui::Vec2::new(4.0, 0.0);
                                        ui.label(format!("{}:", definition.label));
                                        ui.label(RichText::new(format!("{:.1} ms", average * 1000.0)).color(definition.color));
                                    });

                                    plot_data.push((
                                        definition.color,
                                        (0..collector.delta_history_capacity)
                                            .map(|i| PlotPoint::new(i as f32, *data.history.get(i).unwrap_or(&0.0) * 1000.0))
                                            .collect::<Vec<PlotPoint>>(),
                                    ));
                                }
                            }
                        });
                    });

                    ui[3].vertical(|ui| {
                        let plot = Plot::new("ProfilerPlot")
                            .height(100.0)
                            .auto_bounds_x()
                            .auto_bounds_y()
                            .set_margin_fraction(Vec2::new(0.0, 0.1))
                            .include_y(0.0)
                            .include_y(10.0)
                            .allow_zoom(false)
                            .allow_drag(false)
                            .allow_scroll(false)
                            .allow_double_click_reset(false)
                            .allow_boxed_zoom(false)
                            .show_x(false)
                            .show_y(false)
                            .y_axis_width(1)
                            .x_axis_formatter(|_, _, _| "".to_string())
                            .show_grid(Vec2b::new(false, true));

                        plot.show(ui, |plot_ui| {
                            for (color, data) in plot_data {
                                plot_ui.line(Line::new(PlotPoints::Owned(data)).color(color).style(LineStyle::Solid));
                            }
                        });
                    });
                });

                //ui.add_space(10.0);
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(format!("Draw calls: {} ({} tris)", data.renderer.draw_calls, data.renderer.triangles));
                        });
                        ui.horizontal(|ui| {
                            ui.label(format!("Sprite buffer size: {:.1} MB", data.renderer.sprite_buffer_size as f32 / 1024.0 / 1024.0));
                        });
                        ui.horizontal(|ui| {
                            ui.label(format!("Shape buffer size: {:.1} MB", data.renderer.shape_buffer_size as f32 / 1024.0 / 1024.0));
                        });
                        ui.horizontal(|ui| {
                            ui.label(format!(
                                "Loaded textures: {} ({:.1} MB)",
                                data.renderer.loaded_textures_count,
                                data.renderer.loaded_textures_size as f32 / 1024.0 / 1024.0
                            ));
                        });
                    });
                    ui.add_space(10.0);
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.style_mut().spacing.item_spacing = egui::Vec2::new(4.0, 0.0);
                            ui.label("Private memory:");
                            ui.label(RichText::new(format!("{:.1} MB", data.private_memory_current as f32 / 1024.0 / 1024.0)).color(Color32::GREEN));
                        });
                        ui.horizontal(|ui| {
                            ui.style_mut().spacing.item_spacing = egui::Vec2::new(4.0, 0.0);
                            ui.label(format!("Private memory (peak): {:.1} MB", data.private_memory_peak as f32 / 1024.0 / 1024.0));
                        });
                        ui.horizontal(|ui| {
                            ui.style_mut().spacing.item_spacing = egui::Vec2::new(4.0, 0.0);
                            ui.label("Reserved memory:");
                            ui.label(
                                RichText::new(format!("{:.1} MB", data.reserved_memory_current as f32 / 1024.0 / 1024.0)).color(Color32::LIGHT_BLUE),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.style_mut().spacing.item_spacing = egui::Vec2::new(4.0, 0.0);
                            ui.label(format!("Reserved memory (peak): {:.1} MB", data.reserved_memory_peak as f32 / 1024.0 / 1024.0));
                        });
                    });

                    let plot = Plot::new("MemoryPlot")
                        .height(100.0)
                        .auto_bounds_x()
                        .auto_bounds_y()
                        .set_margin_fraction(Vec2::new(0.0, 0.1))
                        .include_y(0.0)
                        .include_y(100.0)
                        .allow_zoom(false)
                        .allow_drag(false)
                        .allow_scroll(false)
                        .allow_double_click_reset(false)
                        .allow_boxed_zoom(false)
                        .show_x(false)
                        .show_y(false)
                        .y_axis_width(3)
                        .x_axis_formatter(|_, _, _| "".to_string())
                        .show_grid(Vec2b::new(false, true));
                    let plot_data_private_memory = (0..collector.memory_history_capacity)
                        .map(|i| PlotPoint::new(i as f32, *collector.private_memory_history.get(i).unwrap_or(&0) as f32 / 1024.0 / 1024.0))
                        .collect::<Vec<PlotPoint>>();
                    let plot_data_reserved_memory = (0..collector.memory_history_capacity)
                        .map(|i| PlotPoint::new(i as f32, *collector.reserved_memory_history.get(i).unwrap_or(&0) as f32 / 1024.0 / 1024.0))
                        .collect::<Vec<PlotPoint>>();

                    plot.show(ui, |plot_ui| {
                        plot_ui.line(Line::new(PlotPoints::Owned(plot_data_private_memory)).color(Color32::GREEN).style(LineStyle::Solid));
                        plot_ui.line(Line::new(PlotPoints::Owned(plot_data_reserved_memory)).color(Color32::LIGHT_BLUE).style(LineStyle::Solid));
                    });
                });

                ui.vertical_centered_justified(|ui| {
                    ScrollArea::vertical().max_height(95.0).show(ui, |ui| {
                        ui.add(TextEdit::multiline(&mut console.output_content).desired_rows(6).interactive(false));

                        if console.is_changed() {
                            ui.scroll_to_cursor(Some(Align::BOTTOM));
                        }
                    });

                    if ui.add(TextEdit::multiline(&mut console.input_content).desired_rows(1)).changed() {
                        if console.input_content.chars().last().unwrap_or('\0') == '\n' {
                            console.apply_input();
                        }
                    }
                });
            });
    }
}
