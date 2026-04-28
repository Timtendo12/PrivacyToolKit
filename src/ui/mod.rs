use eframe::egui;

pub fn nav_button(ui: &mut egui::Ui, label: &str, is_active: bool) -> bool {
    let width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, 48.0), egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let visuals = ui.visuals();
        let bg = if is_active {
            visuals.selection.bg_fill
        } else if response.hovered() {
            visuals.widgets.hovered.weak_bg_fill
        } else {
            egui::Color32::TRANSPARENT
        };

        ui.painter().rect_filled(rect, 6.0, bg);

        let text_color = if is_active {
            visuals.selection.stroke.color
        } else {
            visuals.text_color()
        };

        ui.painter().text(
            rect.left_center() + egui::vec2(14.0, 0.0),
            egui::Align2::LEFT_CENTER,
            label,
            egui::FontId::proportional(16.0),
            text_color,
        );
    }

    response.clicked()
}
