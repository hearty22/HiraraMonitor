use egui::Ui;

use super::theme::{BORDER, CARD_BG, MUTED, TEXT};

/// Draws content inside a consistently styled card.
/// Draws content inside a consistently styled card.
pub fn card(ui: &mut Ui, fill: egui::Color32, content: impl FnOnce(&mut Ui)) {
    egui::Frame::none()
        .fill(fill)
        .stroke(egui::Stroke::new(1.0, BORDER))
        .rounding(egui::Rounding::same(8.0))
        .inner_margin(egui::Margin::same(12.0))
        .show(ui, content);
}

/// Draws a section card with a title.
/// Draws a section card with a title.
pub fn section_card(ui: &mut Ui, title: &str, content: impl FnOnce(&mut Ui)) {
    card(ui, CARD_BG, |ui| {
        ui.label(egui::RichText::new(title).strong().color(TEXT).size(15.0));
        ui.add_space(6.0);
        content(ui);
    });
}

#[allow(dead_code)]
/// Draws a compact metric card.
/// Draws a compact metric card.
pub fn metric_card(ui: &mut Ui, label: &str, value: &str, accent: egui::Color32) {
    card(ui, CARD_BG, |ui| {
        ui.label(egui::RichText::new(label).color(MUTED).size(11.0));
        ui.label(egui::RichText::new(value).strong().color(accent).size(18.0));
    });
}

#[allow(dead_code)]
/// Draws a small colored status badge.
/// Draws a small colored status badge.
pub fn badge(ui: &mut Ui, text: &str, color: egui::Color32) {
    egui::Frame::none()
        .fill(color.linear_multiply(0.18))
        .stroke(egui::Stroke::new(1.0, color))
        .rounding(egui::Rounding::same(10.0))
        .inner_margin(egui::Margin::symmetric(8.0, 3.0))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(text).color(color).size(11.0));
        });
}

#[allow(dead_code)]
/// Returns a CPU or memory alert color for a thresholded value.
/// Returns a CPU or memory alert color for a thresholded value.
pub fn alert_color(
    value: f32,
    warning_threshold: f32,
    critical_threshold: f32,
    enabled: bool,
) -> egui::Color32 {
    if !enabled {
        return super::theme::TEXT;
    }
    if value >= critical_threshold {
        egui::Color32::from_rgb(245, 105, 85)
    } else if value >= warning_threshold {
        egui::Color32::from_rgb(245, 185, 85)
    } else {
        egui::Color32::from_rgb(95, 185, 110)
    }
}
