use egui::{RichText, Slider, Ui};
use egui_plot::{Line, Plot, PlotPoints};
use crate::backend::common_settings::CommonSettings;
use crate::backend::output_settings::{OutsideRange, VelocityCurve};

pub fn velocity_map_settings(ui: &mut Ui, settings: &mut impl CommonSettings, unique_id: String) {
    let points: PlotPoints = (1..127).map(|x| [x as f64, settings.get_velocity(x as f64)]).filter(|[_, y]| *y > 0.0).collect();
    let line = Line::new(points);
    let plot = Plot::new(format!("velocity_curve-{unique_id}"))
        .width(100.0)
        .view_aspect(1.0)
        .include_x(1.0)
        .include_y(1.0)
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
            ui.selectable_value(settings.velocity_curve_mut(), VelocityCurve::Linear, "Linear");
            if ui.selectable_label(matches!(settings.velocity_curve(), VelocityCurve::Fixed(_)), "Fixed").clicked() {
                *settings.velocity_curve_mut() = VelocityCurve::Fixed(64);
            };
            if ui.selectable_label(matches!(settings.velocity_curve(), VelocityCurve::Exponential(_)), "Exponential").clicked() {
                *settings.velocity_curve_mut() = VelocityCurve::Exponential(2.0);
            };
            if ui.selectable_label(matches!(settings.velocity_curve(), VelocityCurve::Logarithmic(_)), "Logarithmic").clicked() {
                *settings.velocity_curve_mut() = VelocityCurve::Logarithmic(2.0);
            };
            if ui.selectable_label(matches!(settings.velocity_curve(), VelocityCurve::SCurve(_)), "S Curve").clicked() {
                *settings.velocity_curve_mut() = VelocityCurve::SCurve(1.0);
            };
        });

        ui.vertical(|ui| {
            match settings.velocity_curve_mut() {
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

            match settings.velocity_curve() {
                VelocityCurve::Fixed(_) => {}
                _ => {
                    ui.label("Minimum:");
                    ui.add(
                        Slider::new(&mut settings.velocity_range_mut().min, 1..=127)
                    );
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Below:").small());
                        ui.selectable_value(&mut settings.velocity_range_mut().below_min, OutsideRange::Ignore, RichText::new("Ignore").small());
                        ui.selectable_value(&mut settings.velocity_range_mut().below_min, OutsideRange::Clamp, RichText::new("Clamp").small());
                        ui.selectable_value(&mut settings.velocity_range_mut().below_min, OutsideRange::Scale, RichText::new("Scale").small());
                    });
                    ui.label("Maximum:");
                    ui.add(
                        Slider::new(&mut settings.velocity_range_mut().max, 1..=127)
                    );
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Above:").small());
                        ui.selectable_value(&mut settings.velocity_range_mut().above_max, OutsideRange::Ignore, RichText::new("Ignore").small());
                        ui.selectable_value(&mut settings.velocity_range_mut().above_max, OutsideRange::Clamp, RichText::new("Clamp").small());
                        ui.selectable_value(&mut settings.velocity_range_mut().above_max, OutsideRange::Scale, RichText::new("Scale").small());
                    });
                }
            }
        })
    });
}