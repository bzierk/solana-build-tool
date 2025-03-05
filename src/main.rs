use eframe::egui;
use std::sync::mpsc::channel;

mod build;
mod model;
mod ui;

use build::scan_programs;
use model::BuildTool;
use ui::render_ui;

fn main() -> Result<(), eframe::Error> {
    let programs = scan_programs();
    let (tx, rx) = channel();
    let app = BuildTool {
        programs,
        selected_program: None,
        build_output: String::new(),
        build_rx: rx,
        build_tx: tx,
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size(egui::Vec2::new(800.0, 600.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Solana Build Tool",
        options,
        Box::new(|_cc| Ok(Box::new(app))),
    )
}

impl eframe::App for BuildTool {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        while let Ok(output) = self.build_rx.try_recv() {
            self.build_output.push_str(&output);
            self.build_output.push('\n');
        }
        render_ui(self, ctx, frame);
        ctx.request_repaint();
    }
}
