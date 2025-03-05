# Solana Build Tool

A graphical user interface for building and managing Solana programs with configurable features.

## Overview

Solana Build Tool simplifies the process of building Solana programs with specific feature flags. It provides a user-friendly interface to:

- Scan and detect Solana programs in your workspace
- Select specific features to enable for each program
- Build programs individually or all at once
- Save and load build presets for quick access
- Configure TypeScript IDL output directory

## Installation

### Prerequisites

- Rust and Cargo (latest stable version)
- Solana CLI tools
- Anchor (if working with Anchor programs)

### Building from Source

1. Clone this repository:
```bash
git clone git@github.com:bzierk/anchor-build-tool.git
cd solana-build-tool
```

2. Build and run the application:
```bash
cargo build --release
cargo run --release
```

## Usage

### Main Interface

The main interface displays:
- A list of detected Solana programs
- For each program, a set of available features that can be toggled
- Build buttons for selected programs
- Preset management
- Build output log

### Building Programs

1. **Select a Program**: Click on a program name to select it.
2. **Toggle Features**: Check the features you want to enable for the selected program.
3. **Build**:
   - Click "Build Selected" to build only the currently selected program with its selected features
   - Click "Build All (Prod)" to build all programs with the "prod" feature enabled
   - Click "Build All (Local)" to build all programs with their default features

### Working with Presets

Presets allow you to save and reuse specific configurations of programs and features.

1. **Creating a Preset**:
   - Select the programs and features you want to include
   - Click "Save Preset"
   - Enter a name for your preset
   - Click "Save"

2. **Using a Preset**:
   - Click on a preset name in the presets list
   - The tool will automatically build the programs with the features defined in the preset

Presets are automatically saved to `presets.json` in the application directory.

### Configuration Options

Click the "Options" button to access settings:

- **TypeScript IDL Output Directory**: Configure where TypeScript IDL files will be generated. Click "Browse..." to select a directory using a file explorer.

## Development

This application is built using:
- Rust programming language
- eframe/egui for the graphical user interface
- rfd for native file dialogs

## License

[MIT License](LICENSE)
