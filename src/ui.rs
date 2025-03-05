use eframe::egui;
use std::thread;

use crate::build::run_build;
use crate::model::BuildTool;

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

        if ui.button("Build").clicked() {
            app.build_output.clear();
            let tx = app.build_tx.clone();
            let programs = app.programs.clone();
            thread::spawn(move || {
                run_build(programs, tx);
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
                    ui.label(&app.build_output);
                });
        });
    });
}
