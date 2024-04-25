use egui::{Slider, Ui};
use egui_plot::{Line, Plot, PlotPoints};
use crate::backend::common_settings::CommonSettings;
use crate::backend::output_settings::VelocityCurve;

pub fn velocity_map_settings(ui: &mut Ui, settings: &mut impl CommonSettings, unique_id: String) {
    let curve = settings.velocity_curve_mut();
    let points: PlotPoints = (0..127).map(|x| [x as f64, curve.get_y(x as f64)]).collect();
    let line = Line::new(points);
    let plot = Plot::new(format!("velocity_curve-{unique_id}"))
        .width(100.0)
        .view_aspect(1.0)
        .include_x(0.0)
        .include_y(0.0)
        .include_x(127.0)
        .include_y(127.0)
        .show_x(false)
        .show_y(false)
        .show_axes([false; 2])
        .show_grid([false; 2])
        .allow_zoom(false)
        .allow_scroll(false)
        .allow_double_click_reset(false)
        .allow_boxed_zoom(false)
        .allow_drag(false)
        .clamp_grid(true);

    ui.horizontal(|ui| {
        plot.show(ui, |plot| plot.line(line));

        #[allow(clippy::collapsible_if)]
        ui.vertical(|ui| {
            ui.selectable_value(curve, VelocityCurve::Linear, "Linear");
            if ui.selectable_label(matches!(curve, VelocityCurve::Fixed(_)), "Fixed").clicked() {
                *curve = VelocityCurve::Fixed(64);
            };
            if ui.selectable_label(matches!(curve, VelocityCurve::Exponential(_)), "Exponential").clicked() {
                *curve = VelocityCurve::Exponential(2.0);
            };
            if ui.selectable_label(matches!(curve, VelocityCurve::Logarithmic(_)), "Logarithmic").clicked() {
                *curve = VelocityCurve::Logarithmic(2.0);
            };
            if ui.selectable_label(matches!(curve, VelocityCurve::SCurve(_)), "S Curve").clicked() {
                *curve = VelocityCurve::SCurve(1.0);
            };
        });

        ui.vertical(|ui| {
            match curve {
                VelocityCurve::Linear => {}
                VelocityCurve::Fixed(value) => {
                    ui.label("Velocity:");
                    ui.add(
                        Slider::new(value, 0..=127)
                    );
                }
                VelocityCurve::Exponential(exponent) => {
                    ui.label("Steepness:");
                    ui.add(
                        Slider::new(exponent, 0.1..=5.0).logarithmic(true)
                    );
                }
                VelocityCurve::Logarithmic(alpha) => {
                    ui.label("Steepness:");
                    ui.add(
                        Slider::new(alpha, 0.01..=5.0).logarithmic(true)
                    );
                }
                VelocityCurve::SCurve(alpha) => {
                    ui.label("Steepness:");
                    ui.add(
                        Slider::new(alpha, 0.5..=2.0).logarithmic(true)
                    );
                }
            }
        })
    });
}