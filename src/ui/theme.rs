use egui::Color32;

pub const WINDOW_BG: Color32 = Color32::from_rgb(10, 12, 16);
pub const PANEL_BG: Color32 = Color32::from_rgb(10, 12, 16);
pub const CARD_BG: Color32 = Color32::from_rgb(18, 22, 29);
pub const BUTTON_BG: Color32 = Color32::from_rgb(28, 33, 42);
pub const BUTTON_HOVER: Color32 = Color32::from_rgb(40, 46, 58);
pub const PRIMARY: Color32 = Color32::from_rgb(66, 165, 245);
pub const TEXT: Color32 = Color32::from_rgb(240, 240, 240);
pub const MUTED: Color32 = Color32::from_rgb(180, 180, 180);
pub const BORDER: Color32 = Color32::from_rgb(40, 44, 52);

pub fn apply(ctx: &egui::Context, dark: bool) {
    let mut visuals = if dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };

    if dark {
        visuals.window_fill = WINDOW_BG;
        visuals.panel_fill = PANEL_BG;
        visuals.faint_bg_color = CARD_BG;
        visuals.widgets.noninteractive.bg_fill = CARD_BG;
        visuals.widgets.inactive.bg_fill = BUTTON_BG;
        visuals.widgets.active.bg_fill = PRIMARY;
        visuals.widgets.hovered.bg_fill = BUTTON_HOVER;
        visuals.widgets.noninteractive.fg_stroke.color = MUTED;
        visuals.widgets.inactive.fg_stroke.color = TEXT;
        visuals.selection.bg_fill = PRIMARY;
        visuals.selection.stroke.color = TEXT;
    } else {
        visuals.window_fill = Color32::from_rgb(243, 246, 251);
        visuals.panel_fill = Color32::from_rgb(243, 246, 251);
        visuals.faint_bg_color = Color32::from_rgb(230, 235, 243);
        visuals.widgets.inactive.bg_fill = Color32::from_rgb(223, 230, 240);
        visuals.widgets.active.bg_fill = PRIMARY;
        visuals.widgets.hovered.bg_fill = Color32::from_rgb(210, 220, 235);
    }

    visuals.window_rounding = egui::Rounding::same(6.0);
    visuals.menu_rounding = egui::Rounding::same(6.0);
    ctx.set_visuals(visuals);
}
