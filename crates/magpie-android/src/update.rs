use std::{path::Path, time::SystemTime};

use magpie_engine::update::{
    Channel, Platform, Repo, Request, Schedule, Status, Version, check, fetch,
};

const SCHEDULE_FILE: &str = "update-check";
const ABI_SEPARATOR: char = ',';

fn schedule(data_dir: &Path) -> Schedule {
    Schedule::at(data_dir.join(SCHEDULE_FILE))
}

fn abis(list: &str) -> Vec<String> {
    list.split(ABI_SEPARATOR)
        .map(str::trim)
        .filter(|abi| !abi.is_empty())
        .map(str::to_owned)
        .collect()
}

fn status(version: &str, abis: &[String], channel: Channel) -> Status {
    let Some(current) = Version::parse(version) else {
        return Status::Failed {
            message: format!("Version {version} isn't a release version"),
        };
    };
    let request = Request {
        repo: &Repo::default(),
        current: &current,
        platform: Platform::Android { abis },
        channel,
    };
    check(&request, fetch)
}

pub fn due(data_dir: &Path) -> bool {
    schedule(data_dir).due(SystemTime::now())
}

pub fn latest(data_dir: &Path, version: &str, abi_list: &str, channel: &str) -> String {
    let channel = Channel::parse(channel).unwrap_or_default();
    let status = status(version, &abis(abi_list), channel);
    schedule(data_dir).record(SystemTime::now()).ok();
    serde_json::to_string(&status).expect("update statuses serialize to JSON")
}
