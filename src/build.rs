use cargo_metadata::{Metadata, MetadataCommand};
use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc::Sender;

use crate::model::{Feature, Program};

pub type BuildSender = Sender<String>;

pub fn scan_programs() -> Vec<Program> {
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

pub fn run_build(programs: Vec<Program>, tx: BuildSender) {
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
                        tx.send(format!("Build failed with code {:?}", output.status.code()))
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
}

pub fn build_all(programs: Vec<Program>, tx: BuildSender, use_prod: bool) {
    for program in programs {
        let cmd = if use_prod {
            format!("anchor build -p {} -- --features prod", program.name)
        } else {
            format!("anchor build -p {}", program.name)
        };

        tx.send(format!(
            "Running: {} (from {})",
            cmd,
            program.path.display()
        ))
        .unwrap();

        let output = if use_prod {
            Command::new("anchor")
                .args(["build", "-p", &program.name, "--", "--features", "prod"])
                .current_dir(&program.path)
                .envs(std::env::vars())
                .output()
        } else {
            Command::new("anchor")
                .args(["build", "-p", &program.name])
                .current_dir(&program.path)
                .envs(std::env::vars())
                .output()
        };

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
                    tx.send(format!("Build failed with code {:?}", output.status.code()))
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
    tx.send("Build complete.".to_string()).unwrap();
}
