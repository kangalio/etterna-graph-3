use chrono::{Datelike as _, Timelike as _};

// Because of PyO3's bikeshedding (#884), we have to reimplement this shit ourselves -.-
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DateTime(pub chrono::NaiveDateTime);

impl pyo3::ToPyObject for DateTime {
	fn to_object(&self, py: pyo3::Python) -> pyo3::PyObject {
		pyo3::types::PyDateTime::new(
			py,
			self.0.year(),
			self.0.month() as u8,
			self.0.day() as u8,
			self.0.hour() as u8,
			self.0.minute() as u8,
			self.0.second() as u8,
			self.0.timestamp_subsec_micros(),
			None,
		)
		.unwrap()
		.to_object(py)
	}
}

// Yay for PyO3's overengineered trait mess
impl pyo3::IntoPy<pyo3::PyObject> for DateTime {
	fn into_py(self, py: pyo3::Python) -> pyo3::PyObject {
		pyo3::ToPyObject::to_object(&self, py)
	}
}

// Date is commented out because pyqtgraph can't handle raw dates, so better not use that
/*#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date(pub chrono::NaiveDate);

impl pyo3::ToPyObject for Date {
	fn to_object(&self, py: pyo3::Python) -> pyo3::PyObject {
		pyo3::types::PyDate::new(py, self.0.year(), self.0.month() as u8, self.0.day() as u8)
			.unwrap()
			.to_object(py)
	}
}

// Yay for PyO3's overengineered trait mess
impl pyo3::IntoPy<pyo3::PyObject> for Date {
	fn into_py(self, py: pyo3::Python) -> pyo3::PyObject {
		pyo3::ToPyObject::to_object(&self, py)
	}
}*/
