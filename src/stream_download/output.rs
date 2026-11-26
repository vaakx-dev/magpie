use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Progress {
    pub received_bytes: u64,
    pub total_bytes: Option<u64>,
    pub file: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Line {
    Progress(Progress),
    Saved(PathBuf),
    Failure(String),
}

const PROGRESS_MARK: &str = "magpie-progress|";
const SAVED_MARK: &str = "magpie-saved|";
const ERROR_MARK: &str = "ERROR: ";
const FIELD_SEPARATOR: char = '|';
const PROGRESS_FIELDS: usize = 4;
const EXPLANATIONS: [(&str, &str); 2] = [
    ("DRM", "This video is DRM-protected, so it can't be saved."),
    (
        "Unsupported URL",
        "Magpie couldn't find a video on this page.",
    ),
];

pub fn progress_template() -> String {
    format!(
        "download:{PROGRESS_MARK}%(progress.downloaded_bytes)s|%(progress.total_bytes)s|%(progress.total_bytes_estimate)s|%(progress.filename)s"
    )
}

pub fn saved_template() -> String {
    format!("after_move:{SAVED_MARK}%(filepath)s")
}

fn bytes(field: &str) -> Option<u64> {
    let value = field.trim().parse::<f64>().ok()?;
    (value.is_finite() && value >= 0.0).then_some(value as u64)
}

fn progress(fields: &str) -> Option<Progress> {
    let mut fields = fields.splitn(PROGRESS_FIELDS, FIELD_SEPARATOR);
    let (Some(received), Some(total), Some(estimate), Some(file)) =
        (fields.next(), fields.next(), fields.next(), fields.next())
    else {
        return None;
    };
    Some(Progress {
        received_bytes: bytes(received).unwrap_or(0),
        total_bytes: bytes(total).or_else(|| bytes(estimate)),
        file: PathBuf::from(file),
    })
}

fn without_source(message: &str) -> &str {
    let Some((_, after_extractor)) = message.split_once("] ") else {
        return message;
    };
    after_extractor
        .split_once(": ")
        .map_or(after_extractor, |(_, reason)| reason)
}

fn explain(message: &str) -> String {
    let known = EXPLANATIONS
        .iter()
        .find(|(clue, _)| message.contains(clue))
        .map(|(_, explanation)| *explanation);
    known.unwrap_or_else(|| without_source(message)).to_owned()
}

impl Line {
    pub fn parse(text: &str) -> Option<Self> {
        let text = text.trim_end();
        if let Some(fields) = text.strip_prefix(PROGRESS_MARK) {
            return progress(fields).map(Line::Progress);
        }
        if let Some(path) = text.strip_prefix(SAVED_MARK) {
            return Some(Line::Saved(PathBuf::from(path)));
        }
        text.strip_prefix(ERROR_MARK)
            .map(|message| Line::Failure(explain(message)))
    }
}
