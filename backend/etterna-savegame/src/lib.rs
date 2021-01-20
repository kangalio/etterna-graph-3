#[derive(serde::Deserialize, Debug, Clone, PartialEq)]
pub struct XmlData {
	#[serde(rename = "PlayerScores")]
	pub player_scores: PlayerScores,
}

impl XmlData {
	pub fn from_etterna_xml(path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
		let xml = quick_xml::de::from_reader(std::io::BufReader::new(std::fs::File::open(path)?))?;
		Ok(xml)
	}

	pub fn scores_chronologically(&self) -> Vec<&Score> {
		let mut scores = self
			.player_scores
			.charts
			.iter()
			.flat_map(|chart| &chart.scores_at)
			.flat_map(|scores_at| &scores_at.scores)
			.collect::<Vec<_>>();
		scores.sort_by_key(|score| score.datetime);
		scores
	}
}

#[derive(serde::Deserialize, Debug, Clone, PartialEq)]
pub struct PlayerScores {
	#[serde(rename = "Chart", default)]
	pub charts: Vec<Chart>,
}

#[derive(serde::Deserialize, Debug, Clone, PartialEq)]
pub struct Chart {
	#[serde(rename = "ScoresAt", default)]
	pub scores_at: Vec<ScoresAt>,
}

#[derive(serde::Deserialize, Debug, Clone, PartialEq)]
pub struct ScoresAt {
	#[serde(rename = "Score", default)]
	pub scores: Vec<Score>,
}

#[derive(serde::Deserialize, Debug, Clone, PartialEq)]
pub struct Score {
	#[serde(rename = "SkillsetSSRs", default)]
	#[serde(deserialize_with = "deserialize_xml_skillset_ssrs")]
	pub ssr: Option<etterna::Skillsets8>,
	#[serde(rename = "DateTime")]
	#[serde(deserialize_with = "deserialize_xml_datetime")]
	pub datetime: chrono::NaiveDateTime,
	#[serde(rename = "WifeScore")]
	#[serde(deserialize_with = "deserialize_wifescore")]
	pub wifescore_judged: etterna::Wifescore,
	#[serde(rename = "SSRNormPercent")]
	#[serde(deserialize_with = "deserialize_wifescore")]
	pub wifescore_j4: etterna::Wifescore,
}

fn deserialize_xml_skillset_ssrs<'de, D: serde::Deserializer<'de>>(
	deserializer: D,
) -> Result<Option<etterna::Skillsets8>, D::Error> {
	#[derive(serde::Deserialize)]
	#[allow(non_snake_case)]
	struct SkillsetSSRs {
		Overall: f32,
		Stream: f32,
		Jumpstream: f32,
		Handstream: f32,
		Stamina: f32,
		JackSpeed: f32,
		Chordjack: f32,
		Technical: f32,
	}
	let ssrs: Option<SkillsetSSRs> = serde::Deserialize::deserialize(deserializer)?;
	if let Some(ssrs) = ssrs {
		Ok(Some(etterna::Skillsets8 {
			overall: ssrs.Overall,
			stream: ssrs.Stream,
			jumpstream: ssrs.Jumpstream,
			handstream: ssrs.Handstream,
			stamina: ssrs.Stamina,
			jackspeed: ssrs.JackSpeed,
			chordjack: ssrs.Chordjack,
			technical: ssrs.Technical,
		}))
	} else {
		Ok(None)
	}
}

fn deserialize_xml_datetime<'de, D: serde::Deserializer<'de>>(
	deserializer: D,
) -> Result<chrono::NaiveDateTime, D::Error> {
	let datetime: std::borrow::Cow<'static, str> = serde::Deserialize::deserialize(deserializer)?;
	chrono::NaiveDateTime::parse_from_str(&datetime, "%Y-%m-%d %H:%M:%S")
		.map_err(serde::de::Error::custom)
}

fn deserialize_wifescore<'de, D: serde::Deserializer<'de>>(
	deserializer: D,
) -> Result<etterna::Wifescore, D::Error> {
	let wifescore: f32 = serde::Deserialize::deserialize(deserializer)?;
	etterna::Wifescore::from_proportion(wifescore)
		.ok_or_else(|| serde::de::Error::custom(format!("Invalid wifescore: {}", wifescore)))
}
