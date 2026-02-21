mod app;
mod syntax;
mod ui;

use app::SublimeRustApp;
use eframe::egui;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 768.0])
            .with_title("SuRuC"),
        ..Default::default()
    };
    eframe::run_native(
        "sublime_rust_cpu",
        native_options,
        Box::new(|cc| Box::new(SublimeRustApp::new(cc))),
    )
}
