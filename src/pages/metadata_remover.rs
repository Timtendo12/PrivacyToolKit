use std::path::{Path, PathBuf};

use eframe::egui;

#[derive(Default)]
pub struct MetadataRemoverState {
    input_path: Option<PathBuf>,
    temp_output: Option<PathBuf>,
    status: Status,
    preview_texture: Option<egui::TextureHandle>,
}

#[derive(Default)]
enum Status {
    #[default]
    Idle,
    Done,
    Saved(PathBuf),
    Error(String),
}

pub fn show(ui: &mut egui::Ui, state: &mut MetadataRemoverState) {
    egui::Panel::bottom("metadata_controls")
        .resizable(false)
        .show_inside(ui, |ui| {
            ui.add_space(12.0);
            ui.heading("Metadata Remover");
            ui.add_space(4.0);
            ui.label("Strip metadata from images before sharing them.");
            ui.add_space(12.0);

            if ui.button("Upload Image").clicked()
                && let Some(path) = rfd::FileDialog::new()
                    .add_filter("Images", &["jpg", "jpeg", "png", "webp", "bmp", "tiff"])
                    .pick_file()
            {
                state.preview_texture = load_texture(ui.ctx(), &path).ok();
                state.input_path = Some(path);
                state.temp_output = None;
                state.status = Status::Idle;
            }

            if let Some(input) = state.input_path.clone() {
                ui.add_space(4.0);
                ui.label(format!("Selected: {}", input.display()));
                ui.add_space(4.0);
                if ui.button("Clean Image").clicked() {
                    match clean_image(&input) {
                        Ok(temp_path) => {
                            state.temp_output = Some(temp_path);
                            state.status = Status::Done;
                        }
                        Err(e) => {
                            state.status = Status::Error(e);
                        }
                    }
                }
            }

            match &state.status {
                Status::Idle => {}
                Status::Error(e) => {
                    ui.add_space(4.0);
                    ui.colored_label(egui::Color32::RED, format!("Error: {e}"));
                }
                Status::Done => {
                    ui.add_space(4.0);
                    ui.label("Image cleaned successfully.");
                    ui.add_space(4.0);
                    if ui.button("Save Cleaned Image").clicked() {
                        let input = state.input_path.as_deref().unwrap_or(Path::new("image"));
                        let stem = input
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("image");
                        let ext = input.extension().and_then(|e| e.to_str()).unwrap_or("png");
                        let default_name = format!("{stem}_cleaned.{ext}");
                        if let Some(save_path) = rfd::FileDialog::new()
                            .add_filter("Image", &[ext])
                            .set_file_name(&default_name)
                            .save_file()
                            && let Some(temp) = state.temp_output.clone()
                        {
                            match std::fs::copy(&temp, &save_path) {
                                Ok(_) => state.status = Status::Saved(save_path),
                                Err(e) => state.status = Status::Error(e.to_string()),
                            }
                        }
                    }
                }
                Status::Saved(path) => {
                    let path = path.clone();
                    ui.add_space(4.0);
                    ui.label(format!("Saved to: {}", path.display()));
                    ui.add_space(4.0);
                    if ui.button("Clean Another").clicked() {
                        *state = MetadataRemoverState::default();
                    }
                }
            }

            ui.add_space(12.0);
        });

    egui::CentralPanel::default().show_inside(ui, |ui| {
        let rect = ui.max_rect();

        if let Some(ref texture) = state.preview_texture {
            let tex_size = texture.size_vec2();
            let scale = (rect.width() / tex_size.x).min(rect.height() / tex_size.y);
            let display_size = tex_size * scale;
            let display_rect = egui::Rect::from_center_size(rect.center(), display_size);
            ui.painter().image(
                texture.id(),
                display_rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );
        } else {
            let placeholder = rect.shrink(24.0);
            let painter = ui.painter();
            painter.rect_filled(placeholder, 4.0, egui::Color32::from_gray(30));
            painter.text(
                placeholder.center(),
                egui::Align2::CENTER_CENTER,
                "No image uploaded",
                egui::FontId::proportional(14.0),
                egui::Color32::from_gray(100),
            );
        }
    });
}

fn load_texture(ctx: &egui::Context, path: &Path) -> Result<egui::TextureHandle, String> {
    let img = image::open(path).map_err(|e| e.to_string())?;
    let rgba = img.to_rgba8();
    let size = [rgba.width() as usize, rgba.height() as usize];
    let pixels = rgba.into_raw();
    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
    Ok(ctx.load_texture("preview", color_image, egui::TextureOptions::LINEAR))
}

fn clean_image(input: &Path) -> Result<PathBuf, String> {
    let img = image::open(input).map_err(|e| e.to_string())?;
    let format = image::ImageFormat::from_path(input).unwrap_or(image::ImageFormat::Png);
    let temp = std::env::temp_dir().join("privacytoolkit_cleaned.tmp");
    img.save_with_format(&temp, format)
        .map_err(|e| e.to_string())?;
    Ok(temp)
}
