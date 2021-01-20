use pyo3::prelude::*;

use crate::pythrow;

#[pyclass]
#[derive(Debug, Clone, PartialEq)]
pub struct XmlStats {
	#[pyo3(get)]
	xml_path: String,
	#[pyo3(get)]
	ssr_over_time: SsrOverTime,
	#[pyo3(get)]
	acc_over_time: Vec<(crate::DateTime, f32)>,
	#[pyo3(get)]
	skillsets_over_time: Vec<(crate::DateTime, [f32; 8])>,
}

#[pyclass]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SsrOverTime {
	#[pyo3(get)]
	aaaa_and_above: Vec<(crate::DateTime, f32)>,
	#[pyo3(get)]
	aaa: Vec<(crate::DateTime, f32)>,
	#[pyo3(get)]
	aa: Vec<(crate::DateTime, f32)>,
	#[pyo3(get)]
	a: Vec<(crate::DateTime, f32)>,
	#[pyo3(get)]
	b_and_below: Vec<(crate::DateTime, f32)>,
}

pub fn load_xml_stats(xml_path: &str, progress: crate::ProgressHandler) -> PyResult<XmlStats> {
	let mut progress = progress.init(4)?;

	progress.step("Opening Etterna.xml...")?;
	let xml = etterna_savegame::XmlData::from_etterna_xml(xml_path.as_ref()).map_err(pythrow)?;
	let scores = xml.scores_chronologically();

	progress.step("Calculating SSRs over time...")?;
	let mut ssr_over_time = SsrOverTime::default();
	for chart in &xml.player_scores.charts {
		for scores_at in &chart.scores_at {
			for score in &scores_at.scores {
				if let Some(ssr) = &score.ssr {
					let entry = (crate::DateTime(score.datetime), ssr.overall);
					if score.wifescore_j4 >= etterna::Wifescore::AAAA_THRESHOLD {
						ssr_over_time.aaaa_and_above.push(entry);
					} else if score.wifescore_j4 >= etterna::Wifescore::AAA_THRESHOLD {
						ssr_over_time.aaa.push(entry);
					} else if score.wifescore_j4 >= etterna::Wifescore::AA_THRESHOLD {
						ssr_over_time.aa.push(entry);
					} else if score.wifescore_j4 >= etterna::Wifescore::A_THRESHOLD {
						ssr_over_time.a.push(entry);
					} else {
						ssr_over_time.b_and_below.push(entry);
					}
				}
			}
		}
	}

	progress.step("Calculating accuracy over time...")?;
	let acc_over_time = scores
		.iter()
		.map(|score| {
			(
				crate::DateTime(score.datetime),
				score.wifescore_j4.as_proportion(),
			)
		})
		.collect();

	progress.step("Calculating skillsets over time...")?;
	let skill_timeline = etterna::SkillTimeline::calculate(
		scores.iter().filter_map(|score| {
			let ssr = score.ssr.as_ref()?;
			Some((
				crate::DateTime(score.datetime.date().and_hms(0, 0, 0)),
				ssr.to_skillsets7(),
			))
		}),
		false,
	);
	let skillsets_over_time = skill_timeline
		.changes
		.into_iter()
		.map(|(datetime, rating)| {
			(
				datetime,
				[
					rating.overall,
					rating.stream,
					rating.jumpstream,
					rating.handstream,
					rating.stamina,
					rating.jackspeed,
					rating.chordjack,
					rating.technical,
				],
			)
		})
		.collect();

	Ok(XmlStats {
		xml_path: xml_path.to_owned(),
		ssr_over_time,
		acc_over_time,
		skillsets_over_time,
	})
}

#[pyclass]
pub struct AccRatingOverTime {
	#[pyo3(get)]
	normal: Vec<(crate::DateTime, f32)>,
	#[pyo3(get)]
	aaa: Vec<(crate::DateTime, f32)>,
	#[pyo3(get)]
	aaaa: Vec<(crate::DateTime, f32)>,
}

/// Returns three graph coordinate lists, for (in order): overall rating, AAA rating, AAAA rating
pub fn calculate_acc_rating_over_time(
	stats: &XmlStats,
	progress: crate::ProgressHandler,
) -> PyResult<AccRatingOverTime> {
	let xml =
		etterna_savegame::XmlData::from_etterna_xml(stats.xml_path.as_ref()).map_err(pythrow)?;
	let scores = xml.scores_chronologically();

	let iter_scores_with_threshold = |threshold| -> Vec<(crate::DateTime, f32)> {
		let skill_timeline = etterna::SkillTimeline::calculate(
			scores.iter().filter_map(|score| {
				if score.wifescore_j4 < threshold {
					return None;
				}
				let ssr = score.ssr.as_ref()?;

				Some((score.datetime.date(), ssr.to_skillsets7()))
			}),
			false,
		);
		skill_timeline
			.changes
			.into_iter()
			.map(|(date, rating)| (crate::DateTime(date.and_hms(0, 0, 0)), rating.overall))
			.collect()
	};

	let normal = stats
		.skillsets_over_time
		.iter()
		.map(|&(dt, [overall, ..])| (dt, overall))
		.collect();

	let mut progress = progress.init(2)?;
	progress.step("Calculating AAA-only rating...")?;
	let aaa = iter_scores_with_threshold(etterna::Wifescore::AAA_THRESHOLD);
	progress.step("Calculating AAAA-only rating...")?;
	let aaaa = iter_scores_with_threshold(etterna::Wifescore::AAAA_THRESHOLD);

	Ok(AccRatingOverTime { normal, aaa, aaaa })
}
