mod checks;
mod render;
mod types;

use crate::error::Result;
use crate::output::{print_json, OutputMode};

use types::DoctorReport;

pub fn run(output: OutputMode) -> Result<()> {
    let report = collect()?;
    match output {
        OutputMode::Json => print_json(&report),
        OutputMode::Text => {
            render::render_text(&report);
            Ok(())
        }
    }
}

pub fn collect() -> Result<DoctorReport> {
    checks::collect()
}
