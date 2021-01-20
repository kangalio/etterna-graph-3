use pyo3::prelude::*;

#[pyclass]
#[derive(Debug, Clone, PartialEq)]
pub struct ReplaysAnalysis {}

pub fn load_replays_analysis(progress: crate::ProgressHandler) -> PyResult<ReplaysAnalysis> {
	const STEPS: u32 = 8;
	let mut progress = progress.init(STEPS)?;
	for i in 0..STEPS {
		progress.step(&format!("We're now at number {}", i))?;
		std::thread::sleep(std::time::Duration::from_millis(100));
	}
	Ok(ReplaysAnalysis {})
}
