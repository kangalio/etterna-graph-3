#![allow(clippy::tabs_in_doc_comments)]

mod progress_callback;

use std::path::PathBuf;

use pyo3::prelude::*;

use progress_callback::ProgressHandler;

fn load_xml_stats(xml_path: &str, progress: ProgressHandler) -> PyResult<XmlStats> {
	const STEPS: u32 = 8;
	let mut progress = progress.init(STEPS)?;
	for i in 0..STEPS {
		progress.step(&format!("We're now at number {}", i))?;
		std::thread::sleep(std::time::Duration::from_millis(100));
	}
	Ok(XmlStats {
		foo: std::fs::read(xml_path)?.len() as u32 * 100 + 123,
	})
}

fn load_replays_analysis(progress: ProgressHandler) -> PyResult<ReplaysAnalysis> {
	const STEPS: u32 = 8;
	let mut progress = progress.init(STEPS)?;
	for i in 0..STEPS {
		progress.step(&format!("We're now at number {}", i))?;
		std::thread::sleep(std::time::Duration::from_millis(100));
	}
	Ok(ReplaysAnalysis {})
}

fn load_charts_analysis(
	replays: &ReplaysAnalysis,
	progress: ProgressHandler,
) -> PyResult<ChartsAnalysis> {
	const STEPS: u32 = 8;
	let mut progress = progress.init(STEPS)?;
	for i in 0..STEPS {
		progress.step(&format!("We're now at number {}", i))?;
		std::thread::sleep(std::time::Duration::from_millis(100));
	}
	Ok(ChartsAnalysis {})
}

#[pyclass]
#[derive(Debug, Clone, PartialEq)]
pub struct XmlStats {
	#[pyo3(get)] foo: u32,
}

#[pyclass]
#[derive(Debug, Clone, PartialEq)]
pub struct ReplaysAnalysis {

}

#[pyclass]
#[derive(Debug, Clone, PartialEq)]
pub struct ChartsAnalysis {

}

#[pyclass]
#[derive(serde::Deserialize, serde::Serialize)]
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
	#[pyo3(get)] paths: EtternaProfilePaths,
}

#[pymethods]
impl Config {
	#[new]
	pub fn new(paths: EtternaProfilePaths) -> Self {
		Self { paths }
	}

	#[staticmethod]
	pub fn load(config_path: &str) -> PyResult<Self> {
		Ok(serde_json::from_str(&std::fs::read_to_string(config_path)?)	
			.map_err(|e| pyo3::exceptions::PyException::new_err(e.to_string()))?)
	}
}

// #[pyclass]
// #[derive(serde::Deserialize, serde::Serialize)]
// #[derive(Debug, Clone, PartialEq)]
// pub struct PathConfig {
// 	xml: PathBuf,
// 	replays_dir: Option<PathBuf>,
// 	songs_dir: Option<PathBuf>,
// }

// #[pymethods]
// impl PathConfig {
// 	#[new]
// 	pub fn new(
// 		xml: String,
// 		replays_dir: Option<String>,
// 		songs_dir: Option<String>
// 	) -> Self {
// 		Self {
// 			xml: xml.into(),
// 			replays_dir: replays_dir.map(|x| x.into()),
// 			songs_dir: songs_dir.map(|x| x.into())
// 		}
// 	}

// 	pub fn exists(&self) -> bool {
// 		if !self.xml.is_file() { return false; }
// 		if self.replays_dir.as_ref().map(|r| r.is_dir()) == Some(false) { return false; }
// 		if self.songs_dir.as_ref().map(|r| r.is_dir()) == Some(false) { return false; }
// 		true
// 	}
// }

#[pyclass]
#[derive(Debug, Clone, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct EtternaProfilePaths {
	xml: PathBuf,
	replays_dir: PathBuf,
	// TODO: integrate AdditionalSongFolders into this
	songs_dir: PathBuf,
}

#[pymethods]
impl EtternaProfilePaths {
	#[new]
	pub fn new(xml: String, replays_dir: String, songs_dir: String) -> Self {
		Self { xml: xml.into(), replays_dir: replays_dir.into(), songs_dir: songs_dir.into() }
	}
	
	pub fn exists(&self) -> bool {
		self.xml.is_file() && self.replays_dir.is_dir() && self.songs_dir.is_dir()
	}

	#[getter]
	fn xml(&self) -> String { self.xml.to_string_lossy().to_string() }
	#[getter]
	fn replays_dir(&self) -> String { self.replays_dir.to_string_lossy().to_string() }
	#[getter]
	fn songs_dir(&self) -> String { self.songs_dir.to_string_lossy().to_string() }
}

#[pyclass]
#[derive(Debug, Clone, PartialEq)]
pub struct DetectedEtternaProfile {
	#[pyo3(get)] paths: EtternaProfilePaths,
	base: PathBuf,
	#[pyo3(get)] xml_size_mb: f32,
	#[pyo3(get)] num_replays: u32,
}

#[pymethods]
impl DetectedEtternaProfile {
	#[getter]
	fn base(&self) -> String { self.base.to_string_lossy().to_string() }
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
		r"C:/Games/Etterna*", // Windows
		r"C:/Users/*/AppData/*/etterna*", // Windows
		r"/home/*/.etterna*", // Linux
		r"/home/*/.stepmania*", // Linux
		r"/opt/etterna*", // Linux
		r"/Users/*/Library/Preferences/Etterna*", // Mac
	] {
		for potential_etterna_base_path in glob::glob_with(glob, CASE_INSENSITIVE_GLOB)? {
			let potential_etterna_base_path = potential_etterna_base_path?;

			let potential_xml_paths = glob::glob_with(
				match potential_etterna_base_path.join("Save/LocalProfiles/*/Etterna.xml").to_str() {
					Some(x) => x,
					None => {
						println!("Skipped non-UTF-8 Etterna.xml path in profile detection!");
						continue;
					},
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
		max_progress: PyObject, progress: PyObject, progress_text: PyObject
	) -> PyResult<XmlStats> {
		py.allow_threads(|| load_xml_stats(
			xml_path,
			ProgressHandler::new(max_progress, progress, progress_text)
		))
	}

	#[pyfn(m, "load_replays_analysis")]
	pub fn load_replays_analysis_py(
		py: Python,
		max_progress: PyObject, progress: PyObject, progress_text: PyObject
	) -> PyResult<ReplaysAnalysis> {
		py.allow_threads(|| load_replays_analysis(
			ProgressHandler::new(max_progress, progress, progress_text)
		))
	}

	#[pyfn(m, "load_charts_analysis")]
	pub fn load_charts_analysis_py(
		py: Python, replays_analysis: &ReplaysAnalysis,
		max_progress: PyObject, progress: PyObject, progress_text: PyObject
	) -> PyResult<ChartsAnalysis> {
		py.allow_threads(|| load_charts_analysis(
			replays_analysis, ProgressHandler::new(max_progress, progress, progress_text)
		))
	}

	#[pyfn(m, "detect_etterna_profiles")]
	pub fn detect_etterna_profiles_py() -> PyResult<Vec<DetectedEtternaProfile>> {
		detect_etterna_profiles()
			.map_err(|e| pyo3::exceptions::PyException::new_err(e.to_string()))
	}

	m.add_class::<ReplaysAnalysis>()?;
	m.add_class::<ChartsAnalysis>()?;
	m.add_class::<EtternaProfilePaths>()?;
	m.add_class::<DetectedEtternaProfile>()?;
	m.add_class::<Config>()?;
	// m.add_class::<PathConfig>()?;
	
	Ok(())
}