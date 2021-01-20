#![allow(clippy::tabs_in_doc_comments)]
// #![allow(unused_imports)] // temporary

mod charts_analysis;
pub use charts_analysis::*;
mod datetime_hack;
pub use datetime_hack::*;
mod progress_callback;
pub use progress_callback::*;
mod replays_analysis;
pub use replays_analysis::*;
mod xml_stats;
pub use xml_stats::*;

use std::path::PathBuf;

use pyo3::prelude::*;

fn pythrow<E: std::fmt::Display>(error: E) -> pyo3::PyErr {
	pyo3::exceptions::PyException::new_err(error.to_string())
}

#[pyclass]
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq)]
pub struct Config {
	#[pyo3(get)]
	paths: EtternaProfilePaths,
}

#[pymethods]
impl Config {
	#[new]
	pub fn new(paths: EtternaProfilePaths) -> Self {
		Self { paths }
	}

	#[staticmethod]
	pub fn load(config_path: &str) -> PyResult<Self> {
		Ok(serde_json::from_str(&std::fs::read_to_string(config_path)?).map_err(pythrow)?)
	}

	pub fn write(&self, config_path: &str) -> PyResult<()> {
		std::fs::write(
			config_path,
			serde_json::to_string_pretty(self).map_err(pythrow)?,
		)?;
		Ok(())
	}
}

#[pyclass]
#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct EtternaProfilePaths {
	#[pyo3(get)]
	xml: PathBuf,
	#[pyo3(get)]
	replays_dir: PathBuf,
	// TODO: integrate AdditionalSongFolders into this
	#[pyo3(get)]
	songs_dir: PathBuf,
}

#[pymethods]
impl EtternaProfilePaths {
	#[new]
	pub fn new(xml: PathBuf, replays_dir: PathBuf, songs_dir: PathBuf) -> Self {
		Self {
			xml,
			replays_dir,
			songs_dir,
		}
	}

	pub fn exists(&self) -> bool {
		self.xml.is_file() && self.replays_dir.is_dir() && self.songs_dir.is_dir()
	}
}

#[pyclass]
#[derive(Debug, Clone, PartialEq)]
pub struct DetectedEtternaProfile {
	#[pyo3(get)]
	paths: EtternaProfilePaths,
	#[pyo3(get)]
	base: PathBuf,
	#[pyo3(get)]
	xml_size_mb: f32,
	#[pyo3(get)]
	num_replays: u32,
}

fn detect_etterna_profiles() -> Result<Vec<DetectedEtternaProfile>, Box<dyn std::error::Error>> {
	const CASE_INSENSITIVE_GLOB: glob::MatchOptions = glob::MatchOptions {
		case_sensitive: false,
		// default values
		require_literal_leading_dot: false,
		require_literal_separator: false,
	};

	let mut detected_profiles = Vec::new();

	for glob in &[
		r"C:/Games/Etterna*",                     // Windows
		r"C:/Users/*/AppData/*/etterna*",         // Windows
		r"/home/*/.etterna*",                     // Linux
		r"/home/*/.stepmania*",                   // Linux
		r"/opt/etterna*",                         // Linux
		r"/Users/*/Library/Preferences/Etterna*", // Mac
	] {
		for potential_etterna_base_path in glob::glob_with(glob, CASE_INSENSITIVE_GLOB)? {
			let potential_etterna_base_path = potential_etterna_base_path?;

			let potential_xml_paths = glob::glob_with(
				match potential_etterna_base_path
					.join("Save/LocalProfiles/*/Etterna.xml")
					.to_str()
				{
					Some(x) => x,
					None => {
						println!("Skipped non-UTF-8 Etterna.xml path in profile detection!");
						continue;
					}
				},
				CASE_INSENSITIVE_GLOB,
			)?;
			for potential_xml_path in potential_xml_paths {
				let paths = EtternaProfilePaths {
					xml: potential_xml_path?,
					replays_dir: potential_etterna_base_path.join("Save/ReplaysV2"),
					songs_dir: potential_etterna_base_path.join("Songs"),
				};
				let potential_etterna_profile = DetectedEtternaProfile {
					base: potential_etterna_base_path.clone(),
					xml_size_mb: std::fs::metadata(&paths.xml)?.len() as f32 / 1_000_000.0,
					num_replays: std::fs::read_dir(&paths.replays_dir)?.count() as u32,
					paths,
				};
				if potential_etterna_profile.paths.exists() {
					detected_profiles.push(potential_etterna_profile);
				}
			}
		}
	}

	Ok(detected_profiles)
}

#[pymodule]
fn backend(_py: Python, m: &PyModule) -> PyResult<()> {
	#[pyfn(m, "load_xml_stats")]
	pub fn load_xml_stats_py(
		py: Python,
		xml_path: &str,
		max_progress: PyObject,
		progress: PyObject,
		progress_text: PyObject,
	) -> PyResult<XmlStats> {
		py.allow_threads(|| {
			load_xml_stats(
				xml_path,
				ProgressHandler::new(max_progress, progress, progress_text),
			)
		})
	}

	#[pyfn(m, "load_replays_analysis")]
	pub fn load_replays_analysis_py(
		py: Python,
		max_progress: PyObject,
		progress: PyObject,
		progress_text: PyObject,
	) -> PyResult<ReplaysAnalysis> {
		py.allow_threads(|| {
			load_replays_analysis(ProgressHandler::new(max_progress, progress, progress_text))
		})
	}

	#[pyfn(m, "load_charts_analysis")]
	pub fn load_charts_analysis_py(
		py: Python,
		replays_analysis: &ReplaysAnalysis,
		max_progress: PyObject,
		progress: PyObject,
		progress_text: PyObject,
	) -> PyResult<ChartsAnalysis> {
		py.allow_threads(|| {
			load_charts_analysis(
				replays_analysis,
				ProgressHandler::new(max_progress, progress, progress_text),
			)
		})
	}

	#[pyfn(m, "calculate_acc_rating_over_time")]
	pub fn calculate_acc_rating_over_time_py(
		py: Python,
		stats: &XmlStats,
		max_progress: PyObject,
		progress: PyObject,
		progress_text: PyObject,
	) -> PyResult<AccRatingOverTime> {
		py.allow_threads(|| {
			calculate_acc_rating_over_time(
				stats,
				ProgressHandler::new(max_progress, progress, progress_text),
			)
		})
	}

	#[pyfn(m, "detect_etterna_profiles")]
	pub fn detect_etterna_profiles_py() -> PyResult<Vec<DetectedEtternaProfile>> {
		detect_etterna_profiles().map_err(|e| pyo3::exceptions::PyException::new_err(e.to_string()))
	}

	m.add_class::<ReplaysAnalysis>()?;
	m.add_class::<ChartsAnalysis>()?;
	m.add_class::<EtternaProfilePaths>()?;
	m.add_class::<DetectedEtternaProfile>()?;
	m.add_class::<Config>()?;
	m.add_class::<AccRatingOverTime>()?;

	Ok(())
}
