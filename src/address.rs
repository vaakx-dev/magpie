use url::{Host, Url};

use crate::{config::Search, internal::Page};

pub enum Destination {
    Address(Url),
    Search(Url),
}

const OPENABLE_SCHEMES: [&str; 4] = ["http", "https", "data", "file"];
const LOCAL_HOSTS: [&str; 2] = ["localhost", "127.0.0.1"];
const LOCAL_DOMAIN: &str = "localhost";
const LOCAL_DOMAIN_SUFFIX: &str = ".localhost";

fn host_like(input: &str) -> Option<Url> {
    if input.contains(char::is_whitespace) {
        return None;
    }
    let local = LOCAL_HOSTS.iter().any(|host| input.starts_with(host));
    match (local, input.contains('.')) {
        (true, _) => Url::parse(&format!("http://{input}")).ok(),
        (false, true) => Url::parse(&format!("https://{input}")).ok(),
        (false, false) => None,
    }
}

fn openable(url: &Url) -> bool {
    OPENABLE_SCHEMES.contains(&url.scheme()) || Page::of(url).is_some()
}

impl Destination {
    pub fn url(self) -> Url {
        match self {
            Destination::Address(url) | Destination::Search(url) => url,
        }
    }
}

pub fn address(input: &str) -> Option<Url> {
    let input = input.trim();
    let explicit = Url::parse(input).ok().filter(openable);
    explicit.or_else(|| host_like(input))
}

pub fn destination(input: &str, search: &Search) -> Destination {
    match address(input) {
        Some(url) => Destination::Address(url),
        None => Destination::Search(search.url(input.trim())),
    }
}

fn local(host: Host<&str>) -> bool {
    match host {
        Host::Domain(domain) => domain == LOCAL_DOMAIN || domain.ends_with(LOCAL_DOMAIN_SUFFIX),
        Host::Ipv4(ip) => ip.is_loopback() || ip.is_private() || ip.is_link_local(),
        Host::Ipv6(ip) => ip.is_loopback(),
    }
}

pub fn upgraded(url: &Url) -> Option<Url> {
    let insecure = url.scheme() == "http" && !url.host().is_some_and(local);
    let mut secure = url.clone();
    (insecure && secure.set_scheme("https").is_ok()).then_some(secure)
}
