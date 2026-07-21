// Build script for the TouchGrass Slint prototype.
//
// The only job of this script is to invoke `slint_build::compile`
// on the placeholder UI file once per build. The Slint compiler
// writes the generated component types next to the .slint file,
// where `src/ui/mod.rs` picks them up via `slint::include_modules!()`.
//
// All other module trees under `src/` are pure Rust and are
// handled by the standard cargo build pipeline.

const UI_FILE: &str = "src/ui/preview.slint";

fn main() {
    slint_build::compile(UI_FILE).expect("failed to compile UI file");
}
