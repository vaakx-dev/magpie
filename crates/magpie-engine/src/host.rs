pub fn within(host: &str, site: &str) -> bool {
    host == site
        || host
            .strip_suffix(site)
            .is_some_and(|subdomain| subdomain.ends_with('.'))
}
