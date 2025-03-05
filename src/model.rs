use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};

#[derive(Clone)]
pub struct Feature {
    pub name: String,
    pub sub_features: Vec<String>,
}

#[derive(Clone)]
pub struct Program {
    pub name: String,
    pub features: Vec<Feature>,
    pub selected: Vec<bool>,
    pub path: PathBuf,
}

pub struct BuildTool {
    pub programs: Vec<Program>,
    pub selected_program: Option<usize>,
    pub build_output: String,
    pub build_rx: Receiver<String>,
    pub build_tx: Sender<String>,
    pub build_dir: Option<String>, // New field for -t flag
}
