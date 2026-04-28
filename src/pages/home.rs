use eframe::egui;

pub fn show(ui: &mut egui::Ui) {
    ui.add_space(24.0);
    ui.heading("Welcome to Privacy Toolkit");
    ui.add_space(4.0);
    ui.label("Your all-in-one toolkit for taking back control of your privacy.");
    ui.add_space(20.0);
    ui.separator();
    ui.add_space(16.0);

    ui.label("Select a tool from the sidebar to get started.");
    ui.add_space(12.0);
    ui.label("More tools are coming soon.");
}
