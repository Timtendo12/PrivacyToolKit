use std::path::{Path, PathBuf};
use std::sync::mpsc;

use eframe::egui;

#[derive(Clone, Copy, PartialEq)]
enum FileKind {
    Image,
    Video,
}

pub struct MetadataRemoverState {
    input_path: Option<PathBuf>,
    temp_output: Option<PathBuf>,
    status: Status,
    preview_texture: Option<egui::TextureHandle>,
    file_kind: Option<FileKind>,
    worker_rx: Option<mpsc::Receiver<Result<PathBuf, String>>>,
    /// Cached result of the FFmpeg availability check. None = not yet checked.
    ffmpeg_available: Option<bool>,
}

impl Default for MetadataRemoverState {
    fn default() -> Self {
        Self {
            input_path: None,
            temp_output: None,
            status: Status::Idle,
            preview_texture: None,
            file_kind: None,
            worker_rx: None,
            ffmpeg_available: None,
        }
    }
}

#[derive(Default)]
enum Status {
    #[default]
    Idle,
    Processing,
    Done,
    Saved(PathBuf),
    Error(String),
}

const IMAGE_EXTS: &[&str] = &["jpg", "jpeg", "png", "webp", "bmp", "tiff"];
const VIDEO_EXTS: &[&str] = &["mp4", "mov", "mkv", "avi", "webm", "m4v", "flv", "wmv", "ts", "mts"];
const ALL_EXTS: &[&str] = &[
    "jpg", "jpeg", "png", "webp", "bmp", "tiff",
    "mp4", "mov", "mkv", "avi", "webm", "m4v", "flv", "wmv", "ts", "mts",
];

fn detect_kind(path: &Path) -> Option<FileKind> {
    let ext = path.extension()?.to_str()?.to_lowercase();
    if IMAGE_EXTS.contains(&ext.as_str()) {
        Some(FileKind::Image)
    } else if VIDEO_EXTS.contains(&ext.as_str()) {
        Some(FileKind::Video)
    } else {
        None
    }
}

pub fn show(ui: &mut egui::Ui, state: &mut MetadataRemoverState) {
    // Poll background worker each frame while processing.
    if matches!(state.status, Status::Processing) {
        if let Some(rx) = &state.worker_rx {
            match rx.try_recv() {
                Ok(Ok(temp_path)) => {
                    state.temp_output = Some(temp_path);
                    state.status = Status::Done;
                    state.worker_rx = None;
                }
                Ok(Err(e)) => {
                    state.status = Status::Error(e);
                    state.worker_rx = None;
                }
                Err(mpsc::TryRecvError::Empty) => {
                    ui.ctx().request_repaint();
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    state.status =
                        Status::Error("Processing thread terminated unexpectedly.".to_string());
                    state.worker_rx = None;
                }
            }
        }
    }

    egui::Panel::bottom("metadata_controls")
        .resizable(false)
        .show_inside(ui, |ui| {
            ui.add_space(12.0);
            ui.heading("Metadata Remover");
            ui.add_space(4.0);
            ui.label("Strip metadata from images and videos before sharing them.");
            ui.add_space(12.0);

            if ui.button("Upload File").clicked()
                && let Some(path) = rfd::FileDialog::new()
                    .add_filter("Images & Videos", ALL_EXTS)
                    .add_filter("Images", IMAGE_EXTS)
                    .add_filter("Videos", VIDEO_EXTS)
                    .pick_file()
            {
                let kind = detect_kind(&path);
                state.preview_texture = if kind == Some(FileKind::Image) {
                    load_texture(ui.ctx(), &path).ok()
                } else {
                    None
                };
                state.input_path = Some(path);
                state.temp_output = None;
                state.status = Status::Idle;
                state.file_kind = kind;
                state.worker_rx = None;
            }

            if let Some(input) = state.input_path.clone() {
                ui.add_space(4.0);
                ui.label(format!("Selected: {}", input.display()));
                ui.add_space(4.0);

                // Lazy FFmpeg check: run once when a video is first loaded.
                if state.file_kind == Some(FileKind::Video)
                    && state.ffmpeg_available.is_none()
                {
                    state.ffmpeg_available = Some(is_ffmpeg_available());
                }

                let ffmpeg_missing = state.file_kind == Some(FileKind::Video)
                    && state.ffmpeg_available != Some(true);
                let can_clean =
                    !matches!(state.status, Status::Processing) && !ffmpeg_missing;

                let response = ui.add_enabled(can_clean, egui::Button::new("Clean File"));

                if ffmpeg_missing {
                    response.on_disabled_hover_text(
                        "FFmpeg is not installed or not found in PATH.\n\
                         Install FFmpeg to clean video files.",
                    );
                } else if response.clicked() {
                    let path = input.clone();
                    let kind = state.file_kind;
                    let (tx, rx) = mpsc::channel();
                    state.worker_rx = Some(rx);
                    state.status = Status::Processing;
                    std::thread::spawn(move || {
                        let result = match kind {
                            Some(FileKind::Video) => clean_video(&path),
                            _ => clean_image(&path),
                        };
                        let _ = tx.send(result);
                    });
                }
            }

            match &state.status {
                Status::Idle => {}
                Status::Processing => {
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("Processing… this may take a while for large files.");
                    });
                }
                Status::Error(e) => {
                    ui.add_space(4.0);
                    ui.colored_label(egui::Color32::RED, format!("Error: {e}"));
                }
                Status::Done => {
                    ui.add_space(4.0);
                    let kind_label =
                        if state.file_kind == Some(FileKind::Video) { "Video" } else { "Image" };
                    ui.label(format!("{kind_label} cleaned successfully."));
                    ui.add_space(4.0);
                    if ui.button("Save Cleaned File").clicked() {
                        let input =
                            state.input_path.as_deref().unwrap_or(Path::new("file"));
                        let stem = input
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("file");
                        let ext =
                            input.extension().and_then(|e| e.to_str()).unwrap_or("mp4");
                        let default_name = format!("{stem}_cleaned.{ext}");
                        if let Some(save_path) = rfd::FileDialog::new()
                            .add_filter("File", &[ext])
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

            let text = match (&state.input_path, state.file_kind) {
                (Some(path), Some(FileKind::Video)) => {
                    let name =
                        path.file_name().and_then(|n| n.to_str()).unwrap_or("video");
                    let size_label = std::fs::metadata(path)
                        .map(|m| format_file_size(m.len()))
                        .unwrap_or_default();
                    format!("{name}\n{size_label}")
                }
                (None, _) => "No file uploaded".to_string(),
                _ => "No preview available".to_string(),
            };
            painter.text(
                placeholder.center(),
                egui::Align2::CENTER_CENTER,
                text,
                egui::FontId::proportional(14.0),
                egui::Color32::from_gray(100),
            );
        }
    });
}

fn format_file_size(bytes: u64) -> String {
    const GB: u64 = 1_073_741_824;
    const MB: u64 = 1_048_576;
    const KB: u64 = 1_024;
    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.0} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}

fn load_texture(ctx: &egui::Context, path: &Path) -> Result<egui::TextureHandle, String> {
    let img = image::open(path).map_err(|e| e.to_string())?;
    let rgba = img.to_rgba8();
    let size = [rgba.width() as usize, rgba.height() as usize];
    let pixels = rgba.into_raw();
    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
    Ok(ctx.load_texture("preview", color_image, egui::TextureOptions::LINEAR))
}

fn is_ffmpeg_available() -> bool {
    std::process::Command::new("ffmpeg")
        .arg("-version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn clean_image(input: &Path) -> Result<PathBuf, String> {
    let img = image::open(input).map_err(|e| e.to_string())?;
    let format = image::ImageFormat::from_path(input).unwrap_or(image::ImageFormat::Png);
    let temp = std::env::temp_dir().join("privacytoolkit_cleaned.tmp");
    img.save_with_format(&temp, format).map_err(|e| e.to_string())?;
    Ok(temp)
}

// Strips all metadata from a video using FFmpeg (stream copy, no re-encode).
// FFmpeg must be installed and available on PATH.
fn clean_video(input: &Path) -> Result<PathBuf, String> {
    let ext = input.extension().and_then(|e| e.to_str()).unwrap_or("mp4");
    let temp = std::env::temp_dir().join(format!("privacytoolkit_cleaned_video.{ext}"));

    let output = std::process::Command::new("ffmpeg")
        .args([
            "-y",
            "-i",
            &input.to_string_lossy(),
            "-map_metadata",
            "-1",
            "-c:v",
            "copy",
            "-c:a",
            "copy",
            temp.to_str().ok_or("Invalid temp path")?,
        ])
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                "FFmpeg not found. Please install FFmpeg and ensure it is in your PATH.".to_string()
            } else {
                e.to_string()
            }
        })?;

    if output.status.success() {
        Ok(temp)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // FFmpeg writes progress to stderr even on success; surface only the tail.
        let last = stderr.lines().rev().take(3).collect::<Vec<_>>().into_iter().rev().collect::<Vec<_>>().join("\n");
        Err(format!("FFmpeg error: {last}"))
    }
}
