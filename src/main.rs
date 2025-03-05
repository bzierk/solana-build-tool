use cargo_metadata::{Metadata, MetadataCommand};
use eframe::egui;
use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

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

fn scan_programs() -> Vec<Program> {
    let metadata = MetadataCommand::new()
        .exec()
        .expect("Failed to run cargo metadata");

    let current_dir = std::env::current_dir().expect("Failed to get current directory");

    metadata
        .packages
        .iter()
        .filter(|p| {
            let is_anchor_program = p.dependencies.iter().any(|dep| dep.name == "anchor-lang");
            let manifest_path = PathBuf::from(&p.manifest_path);
            let is_in_current_dir = manifest_path.starts_with(current_dir.clone());
            is_anchor_program && is_in_current_dir
        })
        .map(|p| {
            let manifest_path = PathBuf::from(&p.manifest_path);
            let program_path = manifest_path
                .parent()
                .expect("Failed to get parent directory")
                .to_path_buf();

            let features = p
                .features
                .iter()
                .map(|(name, deps)| Feature {
                    name: name.clone(),
                    sub_features: deps.clone(),
                })
                .collect();

            Program {
                name: p.name.clone(),
                features,
                selected: Vec::new(),
                path: program_path,
            }
        })
        .collect()
}

#[derive(Clone)]
struct Feature {
    name: String,
    sub_features: Vec<String>,
}

#[derive(Clone)]
struct Program {
    name: String,
    features: Vec<Feature>,
    selected: Vec<bool>,
    path: PathBuf,
}

struct BuildTool {
    programs: Vec<Program>,
    selected_program: Option<usize>,
    build_output: String,
    build_rx: Receiver<String>,
    build_tx: Sender<String>,
}

impl eframe::App for BuildTool {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Ok(output) = self.build_rx.try_recv() {
            self.build_output.push_str(&output);
            self.build_output.push('\n');
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Solana Program Build Tool");
                ui.add_space(10.0);
                if ui.button("Refresh").clicked() {
                    let old_programs = self.programs.clone();
                    self.programs = scan_programs();
                    for new_program in &mut self.programs {
                        if let Some(old_program) =
                            old_programs.iter().find(|p| p.name == new_program.name)
                        {
                            new_program.selected = new_program
                                .features
                                .iter()
                                .map(|f| {
                                    old_program
                                        .features
                                        .iter()
                                        .position(|of| of.name == f.name)
                                        .map(|i| {
                                            old_program.selected.get(i).copied().unwrap_or(false)
                                        })
                                        .unwrap_or(false)
                                })
                                .collect();
                        }
                    }
                    self.selected_program = None;
                }
            });
            ui.add_space(10.0);

            ui.columns(2, |columns| {
                columns[0].group(|ui| {
                    ui.label("Programs:");
                    egui::ScrollArea::vertical()
                        .id_salt("program_list")
                        .max_height(ui.available_height() - 50.0)
                        .show(ui, |ui| {
                            for (i, program) in self.programs.iter().enumerate() {
                                if ui
                                    .selectable_label(
                                        self.selected_program == Some(i),
                                        &program.name,
                                    )
                                    .clicked()
                                {
                                    self.selected_program = Some(i);
                                }
                            }
                        });
                });

                columns[1].group(|ui| {
                    ui.label("Features:");
                    if let Some(selected_idx) = self.selected_program {
                        let program = &mut self.programs[selected_idx];
                        if program.selected.len() != program.features.len() {
                            program.selected.resize(program.features.len(), false);
                        }
                        egui::ScrollArea::vertical()
                            .id_salt("feature_list")
                            .max_height(ui.available_height() - 50.0)
                            .show(ui, |ui| {
                                for (i, feature) in program.features.iter().enumerate() {
                                    let mut label = egui::RichText::new(&feature.name);
                                    if !feature.sub_features.is_empty() {
                                        label = label.underline(); // Visual cue for hoverable info
                                    }
                                    let checkbox = ui.checkbox(&mut program.selected[i], label);
                                    if !feature.sub_features.is_empty() {
                                        checkbox.on_hover_text(format!(
                                            "Includes:\n{}",
                                            feature.sub_features.join("\n")
                                        ));
                                    }
                                }
                            });
                    } else {
                        ui.label("Select a program to view features.");
                    }
                });
            });
            ui.add_space(10.0);

            if ui.button("Build").clicked() {
                self.build_output.clear();
                let tx = self.build_tx.clone();
                let programs = self.programs.clone();

                thread::spawn(move || {
                    for program in programs {
                        let selected_features: Vec<String> = program
                            .features
                            .iter()
                            .zip(&program.selected)
                            .filter(|(_, &sel)| sel)
                            .map(|(f, _)| f.name.clone())
                            .collect();

                        if !selected_features.is_empty() {
                            let feature_args = selected_features.join(",");
                            let cmd = format!(
                                "anchor build -p {} -- --features {}",
                                program.name, feature_args
                            );

                            tx.send(format!(
                                "Running: {} (from {})",
                                cmd,
                                program.path.display()
                            ))
                            .unwrap();

                            let output = Command::new("anchor")
                                .args([
                                    "build",
                                    "-p",
                                    &program.name,
                                    "--",
                                    "--features",
                                    &feature_args,
                                ])
                                .current_dir(&program.path)
                                .envs(std::env::vars())
                                .output();

                            match output {
                                Ok(output) => {
                                    let stdout = String::from_utf8_lossy(&output.stdout);
                                    let stderr = String::from_utf8_lossy(&output.stderr);
                                    if !stdout.is_empty() {
                                        tx.send(stdout.to_string()).unwrap();
                                    }
                                    if !stderr.is_empty() {
                                        tx.send(stderr.to_string()).unwrap();
                                    }
                                    if !output.status.success() {
                                        tx.send(format!(
                                            "Build failed with code {:?}",
                                            output.status.code()
                                        ))
                                        .unwrap();
                                    } else {
                                        tx.send("Build succeeded.".to_string()).unwrap();
                                    }
                                }
                                Err(e) => {
                                    tx.send(format!("Command failed: {}", e)).unwrap();
                                }
                            }
                        }
                    }
                    tx.send("Build complete.".to_string()).unwrap();
                });
            }
            ui.add_space(10.0);

            ui.group(|ui| {
                ui.label("Build Output:");
                egui::ScrollArea::vertical()
                    .id_salt("build_output")
                    .max_height(150.0)
                    .auto_shrink([false, true])
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        ui.label(&self.build_output);
                    });
            });
        });

        ctx.request_repaint();
    }
}
