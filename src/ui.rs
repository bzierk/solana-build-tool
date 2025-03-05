use eframe::egui;
use std::thread;

use crate::build::{build_all, build_preset, run_build};
use crate::model::{BuildTool, Preset};
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
                let options_id = egui::Id::new("options_window");
                if ui.button("Options").clicked() {
                    ctx.memory_mut(|mem| mem.data.insert_temp(options_id, true));
                }

                let mut show_window =
                    ctx.memory(|mem| mem.data.get_temp(options_id).unwrap_or(false));
                if show_window {
                    egui::Window::new("Build Options")
                        .resizable(false)
                        .show(ctx, |ui| {
                            ui.label("TypeScript IDL Output Directory (-t):");
                            let current_dir_str = app
                                .build_dir
                                .clone()
                                .unwrap_or_else(|| "Not set".to_string());
                            ui.label(current_dir_str);
                            if ui.button("Browse...").clicked() {
                                if let Some(path) = FileDialog::new()
                                    .set_directory(std::env::current_dir().unwrap_or_default())
                                    .set_title("Select TypeScript IDL Output Directory")
                                    .pick_folder()
                                {
                                    if let Some(path_str) = path.to_str() {
                                        app.build_dir = Some(path_str.to_string());
                                    }
                                }
                            }
                            ui.add_space(10.0);
                            if ui.button("Clear").clicked() {
                                app.build_dir = None;
                            }
                            ui.add_space(10.0);
                            if ui.button("Close").clicked() {
                                show_window = false;
                            }
                        });
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
            if ui.button("Save Preset").clicked() {
                let preset_popup_id = egui::Id::new("preset_popup_window");
                ctx.memory_mut(|mem| mem.data.insert_temp(preset_popup_id, true));
            }

            let mut show_preset_popup = ctx.memory(|mem| {
                mem.data
                    .get_temp(egui::Id::new("preset_popup_window"))
                    .unwrap_or(false)
            });

            if show_preset_popup {
                let preset_name_id = egui::Id::new("preset_name_input");
                let mut preset_name = ctx
                    .data_mut(|data| data.get_temp::<String>(preset_name_id).unwrap_or_default());

                egui::Window::new("Save Preset")
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Preset Name:");
                            let response = ui.text_edit_singleline(&mut preset_name);
                            if response.changed() {
                                ctx.data_mut(|data| {
                                    data.insert_temp(preset_name_id, preset_name.clone());
                                });
                            }
                        });
                        if ui.button("Save").clicked() && !preset_name.is_empty() {
                            let preset_programs: Vec<(String, Vec<String>)> = app
                                .programs
                                .iter()
                                .filter(|p| !p.selected.iter().all(|&s| !s))
                                .map(|p| {
                                    let selected_features = p
                                        .features
                                        .iter()
                                        .zip(&p.selected)
                                        .filter(|(_, &sel)| sel)
                                        .map(|(f, _)| f.name.clone())
                                        .collect::<Vec<String>>();
                                    (p.name.clone(), selected_features)
                                })
                                .collect();
                            if !preset_programs.is_empty() {
                                app.presets.push(Preset {
                                    name: preset_name.clone(),
                                    programs: preset_programs.clone(),
                                });
                                // Save presets to file
                                if let Ok(json) = serde_json::to_string_pretty(&app.presets) {
                                    let _ = std::fs::write("presets.json", json);
                                }
                                println!(
                                    "Saved preset: {} with {:?}",
                                    preset_name, preset_programs
                                );
                            } else {
                                println!(
                                    "No features selected to save for preset: {}",
                                    preset_name
                                );
                            }
                            show_preset_popup = false;
                            ctx.data_mut(|data| {
                                data.remove::<String>(preset_name_id);
                            });
                        }
                        if ui.button("Cancel").clicked() {
                            show_preset_popup = false;
                            ctx.data_mut(|data| {
                                data.remove::<String>(preset_name_id);
                            });
                        }
                    });

                ctx.memory_mut(|mem| {
                    mem.data
                        .insert_temp(egui::Id::new("preset_popup_window"), show_preset_popup)
                });
            }
        });

        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.label("Presets:");
            for preset in app.presets.iter() {
                let button = ui.button(&preset.name);
                let clicked = button.clicked();

                // Add tooltip with preset details
                let details = preset
                    .programs
                    .iter()
                    .map(|(prog, feats)| {
                        if feats.is_empty() {
                            prog.clone()
                        } else {
                            format!("{}: {}", prog, feats.join(", "))
                        }
                    })
                    .collect::<Vec<String>>()
                    .join("\n");

                button.on_hover_text(format!("Contains:\n{}", details));

                if clicked {
                    app.build_output.clear();
                    let tx = app.build_tx.clone();
                    let programs = app.programs.clone();
                    let preset = preset.clone();
                    let build_dir = app.build_dir.clone();
                    thread::spawn(move || {
                        build_preset(preset, programs, tx, build_dir);
                    });
                }
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
