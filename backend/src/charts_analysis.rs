use pyo3::prelude::*;

#[pyclass]
#[derive(Debug, Clone, PartialEq)]
pub struct ChartsAnalysis {}

pub fn load_charts_analysis(
	replays: &crate::ReplaysAnalysis,
	progress: crate::ProgressHandler,
) -> PyResult<ChartsAnalysis> {
	const STEPS: u32 = 8;
	let mut progress = progress.init(STEPS)?;
	for i in 0..STEPS {
		progress.step(&format!("We're now at number {}", i))?;
		std::thread::sleep(std::time::Duration::from_millis(100));
	}
	Ok(ChartsAnalysis {})
}
