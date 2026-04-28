use crate::pages;
use crate::ui::nav_button;
use eframe::egui;

#[derive(Default, PartialEq)]
pub enum Page {
    #[default]
    Home,
    MetadataRemover,
}

#[derive(Default)]
pub struct App {
    pub current_page: Page,
    pub metadata_remover_state: pages::metadata_remover::MetadataRemoverState,
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::left("sidebar")
            .exact_size(180.0)
            .resizable(false)
            .show_inside(ui, |ui| {
                ui.add_space(8.0);
                ui.heading("PrivacyToolKit");
                ui.add_space(4.0);
                ui.separator();
                ui.add_space(8.0);

                if nav_button(ui, "Home", self.current_page == Page::Home) {
                    self.current_page = Page::Home;
                }
                if nav_button(
                    ui,
                    "Metadata Remover",
                    self.current_page == Page::MetadataRemover,
                ) {
                    self.current_page = Page::MetadataRemover;
                }
            });

        egui::CentralPanel::default().show_inside(ui, |ui| match self.current_page {
            Page::Home => pages::home::show(ui),
            Page::MetadataRemover => {
                pages::metadata_remover::show(ui, &mut self.metadata_remover_state)
            }
        });
    }
}
