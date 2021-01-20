/*!
Struct that can be used by CPU intensive functions as a recipient for progress notifications, that
will be displayed in a Qt progress dialog
*/

use pyo3::prelude::*;

pub struct ProgressHandler {
	max_progress: PyObject,
	progress: PyObject,
	progress_text: PyObject,
}

impl ProgressHandler {
	pub fn new(max_progress: PyObject, progress: PyObject, progress_text: PyObject) -> Self {
		Self {
			max_progress,
			progress,
			progress_text,
		}
	}

	pub fn init(self, num_steps: u32) -> PyResult<ProgressCallback> {
		Python::with_gil(|py| self.max_progress.call_method1(py, "emit", (num_steps,)))?;
		Ok(ProgressCallback {
			signals: self,
			current_progress: 0,
			max_progress: num_steps,
		})
	}
}

/// Example:
/// ```rust
/// fn calculate_something_heavy(progress: ProgressHandler) -> PyResult<SomethingHeavy> {
/// 	let mut progress = progress.init(10);
/// 	for i in 0..10 {
/// 		progress.step(&format!("Step {} out of {}", i + 1, 10));
/// 		// [something cpu heavy]
/// 	}
/// 	Ok(SomethingHeavy { ... })
/// }
/// ```
pub struct ProgressCallback {
	signals: ProgressHandler,
	current_progress: u32,
	max_progress: u32,
}

impl ProgressCallback {
	/// Announce the start of a processing step
	pub fn step(&mut self, progress_text: &str) -> PyResult<()> {
		if self.current_progress >= self.max_progress {
			println!(
				"Warning: progress bar is stepping outside bounds ({} >= {})",
				self.current_progress, self.max_progress
			)
		}

		Python::with_gil(|py| {
			self.signals
				.progress
				.call_method1(py, "emit", (self.current_progress,))?;
			self.signals
				.progress_text
				.call_method1(py, "emit", (progress_text,))
		})?;
		self.current_progress += 1;
		Ok(())
	}
}
