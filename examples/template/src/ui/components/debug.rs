use crate::utils::debug::DebugCollector;
use capybara::egui::Color32;
use capybara::egui::Context;
use capybara::egui::Frame;
use capybara::egui::Margin;
use capybara::egui::RichText;
use capybara::egui::Rounding;
use capybara::egui::Stroke;
use capybara::egui::Vec2b;
use capybara::egui::Window;
use capybara::egui_plot::Line;
use capybara::egui_plot::LineStyle;
use capybara::egui_plot::Plot;
use capybara::egui_plot::PlotPoint;
use capybara::egui_plot::PlotPoints;
use capybara::utils::profiler::Profiler;

pub struct ProfilerPlotDefinition<'a> {
    pub name: &'a str,
    pub label: &'a str,
    pub color: Color32,
}

pub fn debug_window(context: &Context, collector: &mut DebugCollector, profiler: &Profiler) {
    let data = collector.get_data();
    let mut plot_data = Vec::new();

    Window::new("Debug window")
        .frame(debug_frame())
        .resizable(false)
        .collapsible(false)
        .title_bar(false)
        .default_width(700.0)
        .show(context, |ui| {
            ui.style_mut().visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, Color32::from_rgb(255, 255, 255));

            ui.label(&data.hardware_info);
            ui.add_space(15.0);
            ui.columns(4, |ui| {
                ui[0].vertical(|ui| {
                    ui.label(format!("FPS current: {:.1}", data.fps_current));
                    ui.label(format!("FPS average: {}", data.fps_average));
                    ui.horizontal(|ui| {
                        ui.style_mut().spacing.item_spacing = capybara::egui::Vec2::new(4.0, 0.0);
                        ui.label("Delta current:");
                        ui.label(RichText::new(format!("{:.1} ms", data.delta_current * 1000.0)).color(Color32::GREEN));
                    });
                    ui.label(format!("Delta average: {:.1} ms", data.delta_average * 1000.0));
                    ui.label(format!("Delta deviation: {:.1} ms", data.delta_deviation * 1000.0));
                });

                ui[1].horizontal(|ui| {
                    let plot = Plot::new("DeltaPlot")
                        .height(100.0)
                        .auto_bounds_x()
                        .auto_bounds_y()
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
                        let plot_definitions = vec![
                            ProfilerPlotDefinition { name: "input", label: "Input average", color: Color32::RED },
                            ProfilerPlotDefinition { name: "fixed", label: "Fixed average", color: Color32::GREEN },
                            ProfilerPlotDefinition { name: "frame", label: "Frame average", color: Color32::BLUE },
                            ProfilerPlotDefinition { name: "ui", label: "UI average", color: Color32::YELLOW },
                        ];

                        for definition in plot_definitions {
                            if let Some(data) = profiler.data.get(definition.name) {
                                let average = data.history.iter().sum::<f32>() / data.history.len() as f32;
                                ui.horizontal(|ui| {
                                    ui.style_mut().spacing.item_spacing = capybara::egui::Vec2::new(4.0, 0.0);
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
        });
}

fn debug_frame() -> Frame {
    Frame::none()
        .inner_margin(Margin::symmetric(10.0, 10.0))
        .stroke(Stroke::new(1.0, Color32::from_rgba_unmultiplied(0, 0, 0, 220)))
        .fill(Color32::from_rgba_unmultiplied(40, 40, 40, 220))
        .rounding(Rounding::same(5.0))
}
