mod app;
mod pages;
mod ui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Privacy Toolkit",
        options,
        Box::new(|_cc| Ok(Box::new(app::App::default()))),
    )
}
