use std::sync::LazyLock;

use magpie_engine::sponsorblock::{self, Client, Config, Request, View};

static CLIENT: LazyLock<Client> = LazyLock::new(Client::default);

pub fn view(config: &str) -> String {
    serde_json::to_string(&View::from(&Config::parse(config)))
        .expect("SponsorBlock settings serialize to JSON")
}

pub fn switched(config: &str, key: &str, on: bool) -> String {
    let mut config = Config::parse(config);
    config.set(key, sponsorblock::switch(on));
    config.text()
}

pub fn resumed(config: &str, channel: &str) -> String {
    let mut config = Config::parse(config);
    config.resume(channel);
    config.text()
}

pub fn answer(config: &str, request: &str) -> Option<String> {
    let request = Request::parse(request)?;
    let reply = sponsorblock::answer(&CLIENT, &Config::parse(config), &request);
    Some(reply.json())
}
