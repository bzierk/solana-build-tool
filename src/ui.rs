use eframe::egui;
use std::thread;

use crate::build::{build_all, run_build};
use crate::model::BuildTool;

// Add this import for the file dialog
use rfd::FileDialog;

pub fn render_ui(app: &mut BuildTool, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("Solana Program Build Tool");
            ui.add_space(10.0);
            if ui.button("Refresh").clicked() {
                let old_programs = app.programs.clone();
                app.programs = crate::build::scan_programs();
                for new_program in &mut app.programs {
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
                                    .map(|i| old_program.selected.get(i).copied().unwrap_or(false))
                                    .unwrap_or(false)
                            })
                            .collect();
                    }
                }
                app.selected_program = None;
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                // Use a static ID for our window
                let options_id = egui::Id::new("options_window");

                if ui.button("Options").clicked() {
                    // Set the window as open when the button is clicked
                    ctx.memory_mut(|mem| mem.data.insert_temp(options_id, true));
                }

                // Check if the window should be shown
                let mut show_window =
                    ctx.memory(|mem| mem.data.get_temp(options_id).unwrap_or(false));

                // Only show the window if it's supposed to be shown
                if show_window {
                    // Instead of using .open(), we'll manually handle the window state
                    egui::Window::new("Build Options")
                        .resizable(false)
                        .show(ctx, |ui| {
                            ui.label("TypeScript IDL Output Directory (-t):");

                            // Get the current directory path for display
                            let current_dir_str = app.build_dir.clone().unwrap_or_else(|| "Not set".to_string());
                            
                            // Display the current directory
                            ui.label(&current_dir_str);
                            
                            // Add a button to open the folder picker
                            if ui.button("Browse...").clicked() {
                                // Open a folder picker dialog
                                if let Some(path) = FileDialog::new()
                                    .set_directory(std::env::current_dir().unwrap_or_default())
                                    .set_title("Select TypeScript IDL Output Directory")
                                    .pick_folder() 
                                {
                                    // Convert the path to a string
                                    if let Some(path_str) = path.to_str() {
                                        // Store the selected directory
                                        app.build_dir = Some(path_str.to_string());
                                    }
                                }
                            }

                            ui.add_space(10.0);
                            if ui.button("Clear").clicked() {
                                app.build_dir = None;
                            }

                            ui.add_space(10.0);
                            if ui.button("Save").clicked() {
                                // The directory is already saved when selected
                                // Mark the window as closed
                                show_window = false;
                            }

                            // Add a close button in the corner
                            if ui.button("X").clicked() {
                                show_window = false;
                            }
                        });

                    // Update the window state in memory
                    ctx.memory_mut(|mem| mem.data.insert_temp(options_id, show_window));
                }
            });
        });
        ui.add_space(10.0);

        ui.columns(2, |columns| {
            columns[0].group(|ui| {
                ui.label("Programs:");
                egui::ScrollArea::vertical()
                    .id_salt("program_list")
                    .max_height(ui.available_height() - 50.0)
                    .show(ui, |ui| {
                        for (i, program) in app.programs.iter().enumerate() {
                            if ui
                                .selectable_label(app.selected_program == Some(i), &program.name)
                                .clicked()
                            {
                                app.selected_program = Some(i);
                            }
                        }
                    });
            });

            columns[1].group(|ui| {
                ui.label("Features:");
                if let Some(selected_idx) = app.selected_program {
                    let program = &mut app.programs[selected_idx];
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
                                    label = label.underline();
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

        ui.horizontal(|ui| {
            if ui.button("Build").clicked() {
                app.build_output.clear();
                let tx = app.build_tx.clone();
                let programs = app.programs.clone();
                let build_dir = app.build_dir.clone();
                thread::spawn(move || {
                    run_build(programs, tx, build_dir);
                });
            }
            if ui.button("Build All (Prod)").clicked() {
                app.build_output.clear();
                let tx = app.build_tx.clone();
                let programs = app.programs.clone();
                let build_dir = app.build_dir.clone();
                thread::spawn(move || {
                    build_all(programs, tx, true, build_dir);
                });
            }
            if ui.button("Build All (Default)").clicked() {
                app.build_output.clear();
                let tx = app.build_tx.clone();
                let programs = app.programs.clone();
                let build_dir = app.build_dir.clone();
                thread::spawn(move || {
                    build_all(programs, tx, false, build_dir);
                });
            }
        });
        ui.add_space(10.0);

        ui.group(|ui| {
            ui.label("Build Output:");
            egui::ScrollArea::vertical()
                .id_salt("build_output")
                .max_height(150.0)
                .auto_shrink([false, true])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    ui.label(&app.build_output);
                });
        });
    });
}
