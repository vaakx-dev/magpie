use std::{
    io::{BufRead, BufReader, Read, Write},
    net::TcpListener,
    sync::{Arc, Mutex},
    thread,
};

use magpie_engine::bitwarden::{
    CipherId, CipherString, Credentials, Error, Kdf, LoginUri, MatchRule, Method, NewLogin, Page,
    Proof, Sealed, SealedLogin, SealedUri, Server, SignIn, Snapshot, Status, SymmetricKey, Totp,
    Vault, password,
};
use serde_json::json;
use zeroize::Zeroizing;

const PASSWORD: &str = "correct horse battery staple";
const EMAIL: &str = "user@example.com";
const PBKDF2: Kdf = Kdf::Pbkdf2 { iterations: 5000 };
const REFERENCE_CIPHER: &str = "2.AAECAwQFBgcICQoLDA0ODw==|jA6XKjkF7eO5umLsuoWWTg==|YKsGEB15/OhL4ad6a2EdLjllA+6nnw4J2Q5x9fXxhys=";

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn user_key_bytes() -> Vec<u8> {
    (1..=64).collect()
}

fn user_key() -> SymmetricKey {
    SymmetricKey::from_bytes(&user_key_bytes()).unwrap()
}

fn reference_key() -> SymmetricKey {
    PBKDF2.master_key(PASSWORD, EMAIL).unwrap().stretch()
}

fn seal(text: &str, key: &SymmetricKey) -> CipherString {
    CipherString::encrypt(text.as_bytes(), key).unwrap()
}

fn sealed_login(id: &str, name: &str, uris: &[(&str, Option<MatchRule>)]) -> Sealed {
    let key = user_key();
    Sealed {
        id: CipherId(id.to_owned()),
        key: None,
        name: seal(name, &key),
        notes: None,
        favorite: false,
        login: SealedLogin {
            username: Some(seal(&format!("{id}-user"), &key)),
            password: Some(seal(&format!("{id}-secret"), &key)),
            totp: None,
            uris: Some(
                uris.iter()
                    .map(|(uri, rule)| SealedUri {
                        uri: Some(seal(uri, &key)),
                        rule: *rule,
                    })
                    .collect(),
            ),
        },
    }
}

fn protected_key() -> CipherString {
    CipherString::encrypt(&user_key_bytes(), &reference_key()).unwrap()
}

fn snapshot(logins: Vec<Sealed>) -> Snapshot {
    Snapshot {
        server: Server::Europe,
        email: EMAIL.to_owned(),
        kdf: PBKDF2,
        protected_key: protected_key(),
        refresh_token: seal("refresh-token", &user_key()),
        logins,
    }
}

fn unlocked_vault(logins: Vec<Sealed>) -> (tempfile::TempDir, Vault) {
    let folder = tempfile::tempdir().unwrap();
    snapshot(logins).save(folder.path()).unwrap();
    let mut vault = Vault::open(folder.path()).unwrap();
    vault.unlock(PASSWORD).unwrap();
    (folder, vault)
}

fn names(logins: Vec<&magpie_engine::bitwarden::Login>) -> Vec<String> {
    logins.into_iter().map(|login| login.name.clone()).collect()
}

fn uri_matches(uri: &str, rule: MatchRule, address: &str) -> bool {
    let login_uri = LoginUri {
        uri: uri.to_owned(),
        rule,
    };
    login_uri.matches(&Page::parse(address).unwrap())
}

#[test]
fn pbkdf2_master_key_password_hash_and_stretched_key_match_bitwarden() {
    let master_key = PBKDF2.master_key(PASSWORD, "  User@Example.com ").unwrap();
    let stretched = master_key.stretch();
    assert_eq!(
        hex(master_key.as_bytes()),
        "52ce2d33e007a7c15b3e3083c551fcff97da83045fa80cc6bbbd2cbc52c6f084"
    );
    assert_eq!(
        master_key.password_hash(PASSWORD).as_str(),
        "0FMeontUyfpu9Ga/DvERL9LMAXg9KB82VK6UqHdnKko="
    );
    assert_eq!(
        hex(stretched.encryption()),
        "83b79d8c4f57f2e6ab04e4427c3fcc0d9bdeff981254a535a3df2bd2a4df84a7"
    );
    assert_eq!(
        hex(stretched.mac()),
        "016149a9eb912a3d9a41c4b3878a833e3cac88feef99dc4b1a6258716113e881"
    );
}

#[test]
fn argon2id_master_key_matches_bitwarden() {
    let kdf =
        Kdf::from_prelogin(r#"{"kdf":1,"kdfIterations":3,"kdfMemory":16,"kdfParallelism":4}"#)
            .unwrap();
    let master_key = kdf.master_key(PASSWORD, EMAIL).unwrap();
    assert_eq!(
        hex(master_key.as_bytes()),
        "5db398d617288791fb7b7cf1c42ad000acec58bde2c5bed1ca26dc8556f331e0"
    );
}

#[test]
fn prelogin_reads_pbkdf2_and_refuses_settings_a_server_could_use_to_weaken_the_key() {
    assert_eq!(
        Kdf::from_prelogin(r#"{"kdf":0,"kdfIterations":600000,"kdfMemory":null}"#).unwrap(),
        Kdf::Pbkdf2 { iterations: 600000 }
    );
    for weak in [
        r#"{"kdf":0,"kdfIterations":1}"#,
        r#"{"kdf":1,"kdfIterations":3,"kdfMemory":1,"kdfParallelism":4}"#,
        r#"{"kdf":1,"kdfIterations":1,"kdfMemory":64,"kdfParallelism":4}"#,
    ] {
        assert!(
            matches!(Kdf::from_prelogin(weak), Err(Error::WeakKdf)),
            "{weak}"
        );
    }
    assert!(matches!(
        Kdf::from_prelogin(r#"{"kdf":7,"kdfIterations":3}"#),
        Err(Error::Unsupported(_))
    ));
}

#[test]
fn cipher_string_decrypts_aes_cbc_hmac_from_another_implementation() {
    let cipher: CipherString = REFERENCE_CIPHER.parse().unwrap();
    assert_eq!(cipher.decrypt_text(&reference_key()).unwrap(), "hunter2");
    assert_eq!(cipher.to_string(), REFERENCE_CIPHER);
    let round_trip = seal("päss wörd", &user_key());
    let reparsed: CipherString = round_trip.to_string().parse().unwrap();
    assert_eq!(reparsed.decrypt_text(&user_key()).unwrap(), "päss wörd");
    assert_ne!(round_trip, seal("päss wörd", &user_key()));
}

#[test]
fn generated_passwords_are_long_mix_every_class_and_differ() {
    let alphabet = password::CLASSES.concat();
    let generated: Vec<_> = (0..20).map(|_| password::generate().unwrap()).collect();
    for candidate in &generated {
        assert_eq!(candidate.chars().count(), password::LENGTH);
        assert!(
            candidate
                .chars()
                .all(|character| alphabet.contains(character))
        );
        for class in password::CLASSES {
            assert!(candidate.chars().any(|character| class.contains(character)));
        }
    }
    assert_ne!(generated[0], generated[1]);
}

#[test]
fn cipher_string_with_a_tampered_mac_ciphertext_or_wrong_key_is_rejected() {
    let (head, _) = REFERENCE_CIPHER.rsplit_once('|').unwrap();
    let forged_mac: CipherString = format!("{head}|{}", "A".repeat(43) + "=").parse().unwrap();
    let forged_data: CipherString = REFERENCE_CIPHER
        .replace("jA6XKjkF7eO5umLsuoWWTg==", "kA6XKjkF7eO5umLsuoWWTg==")
        .parse()
        .unwrap();
    let original: CipherString = REFERENCE_CIPHER.parse().unwrap();
    assert!(matches!(
        forged_mac.decrypt(&reference_key()),
        Err(Error::Tampered)
    ));
    assert!(matches!(
        forged_data.decrypt(&reference_key()),
        Err(Error::Tampered)
    ));
    assert!(matches!(
        original.decrypt(&user_key()),
        Err(Error::Tampered)
    ));
}

#[test]
fn cipher_string_refuses_other_encryption_types_and_broken_text() {
    assert!(matches!(
        "0.AAECAwQFBgcICQoLDA0ODw==|jA6XKjkF7eO5umLsuoWWTg==".parse::<CipherString>(),
        Err(Error::Unsupported(_))
    ));
    for broken in ["", "2.", "2.AAAA|BBBB", "2.!!|??|**"] {
        assert!(broken.parse::<CipherString>().is_err(), "{broken}");
    }
}

#[test]
fn server_addresses_normalize_to_identity_and_api_endpoints() {
    for us in [
        "vault.bitwarden.com",
        "https://vault.bitwarden.com/",
        " bitwarden.com ",
    ] {
        assert_eq!(Server::parse(us).unwrap(), Server::UnitedStates, "{us}");
    }
    assert_eq!(
        Server::parse("https://vault.bitwarden.eu/#/login").unwrap(),
        Server::Europe
    );
    assert_eq!(
        Server::UnitedStates.identity(),
        "https://identity.bitwarden.com"
    );
    assert_eq!(Server::Europe.api(), "https://api.bitwarden.eu");

    let hosted = Server::parse("vw.example.org/vault/?x=1#/login").unwrap();
    assert_eq!(hosted.address(), "https://vw.example.org/vault");
    assert_eq!(hosted.identity(), "https://vw.example.org/vault/identity");
    assert_eq!(hosted.api(), "https://vw.example.org/vault/api");
    assert_eq!(
        Server::parse("http://192.168.1.5:8080/").unwrap().api(),
        "http://192.168.1.5:8080/api"
    );
    assert!(matches!(
        Server::parse("ftp://example.org"),
        Err(Error::Address(_))
    ));
    assert!(matches!(Server::parse(""), Err(Error::Address(_))));
}

#[test]
fn totp_matches_rfc_6238_vectors() {
    let sha1 = Totp::parse("GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ").unwrap();
    assert_eq!(sha1.code_at(59).digits, "287082");
    assert_eq!(sha1.code_at(59).remaining_secs, 1);
    assert_eq!(sha1.code_at(1_111_111_109).digits, "081804");
    assert_eq!(sha1.code_at(1_111_111_111).remaining_secs, 29);

    let sha256 = Totp::parse(
        "otpauth://totp/Example:user?secret=GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQGEZA&algorithm=SHA256&digits=8",
    )
    .unwrap();
    assert_eq!(sha256.code_at(59).digits, "46119246");

    let sha512 = Totp::parse(
        "otpauth://totp/x?secret=GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQGEZDGNA&algorithm=SHA512&digits=8",
    )
    .unwrap();
    assert_eq!(sha512.code_at(20_000_000_000).digits, "47863826");
}

#[test]
fn totp_accepts_loose_secrets_and_custom_periods_and_rejects_garbage() {
    let spaced = Totp::parse(" gezd gnbv gy3t qojq gezd gnbv gy3t qojq ").unwrap();
    assert_eq!(spaced.code_at(59).digits, "287082");
    let minute =
        Totp::parse("otpauth://totp/x?secret=GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ&period=60").unwrap();
    assert_eq!(minute.code_at(59).remaining_secs, 1);
    assert_eq!(minute.code_at(59).digits.len(), 6);
    for garbage in [
        "",
        "not base32!",
        "otpauth://totp/x?digits=6",
        "otpauth://totp/x?secret=GEZA&period=0",
    ] {
        assert!(Totp::parse(garbage).is_err(), "{garbage}");
    }
}

#[test]
fn uri_match_rules_follow_bitwarden() {
    assert!(uri_matches(
        "example.com",
        MatchRule::Domain,
        "https://login.example.com/a"
    ));
    assert!(uri_matches(
        "https://shop.example.co.uk",
        MatchRule::Domain,
        "https://example.co.uk/"
    ));
    assert!(!uri_matches(
        "example.co.uk",
        MatchRule::Domain,
        "https://other.co.uk/"
    ));
    assert!(!uri_matches(
        "example.com",
        MatchRule::Domain,
        "https://example.com.evil.net/"
    ));
    assert!(uri_matches(
        "192.168.1.5",
        MatchRule::Domain,
        "http://192.168.1.5:8080/"
    ));
    assert!(!uri_matches(
        "192.168.1.5",
        MatchRule::Domain,
        "http://10.168.1.5/"
    ));

    assert!(uri_matches(
        "example.com",
        MatchRule::Host,
        "https://example.com/x"
    ));
    assert!(!uri_matches(
        "example.com",
        MatchRule::Host,
        "https://www.example.com/"
    ));
    assert!(!uri_matches(
        "example.com",
        MatchRule::Host,
        "https://example.com:8443/"
    ));
    assert!(uri_matches(
        "https://example.com:8443",
        MatchRule::Host,
        "https://example.com:8443/"
    ));

    assert!(uri_matches(
        "https://example.com/app",
        MatchRule::StartsWith,
        "https://example.com/app/login"
    ));
    assert!(!uri_matches(
        "https://example.com/app",
        MatchRule::StartsWith,
        "https://example.com/"
    ));
    assert!(uri_matches(
        "https://example.com/login",
        MatchRule::Exact,
        "https://example.com/login"
    ));
    assert!(!uri_matches(
        "https://example.com/login",
        MatchRule::Exact,
        "https://example.com/login?x"
    ));
    assert!(uri_matches(
        r"^https://(www\.)?EXAMPLE\.com/",
        MatchRule::RegularExpression,
        "https://www.example.com/x"
    ));
    assert!(!uri_matches(
        "(",
        MatchRule::RegularExpression,
        "https://example.com/"
    ));
    assert!(!uri_matches(
        "https://example.com/",
        MatchRule::Never,
        "https://example.com/"
    ));
}

#[test]
fn vault_matches_logins_by_url_and_searches_them() {
    let (_folder, vault) = unlocked_vault(vec![
        sealed_login("mail", "Mail", &[("mail.example.com", None)]),
        sealed_login(
            "bank",
            "Bank",
            &[("https://bank.test/login", Some(MatchRule::Exact))],
        ),
        sealed_login(
            "hidden",
            "Hidden",
            &[("example.com", Some(MatchRule::Never))],
        ),
    ]);
    assert_eq!(
        names(vault.matches_for_url("https://www.example.com/inbox")),
        ["Mail"]
    );
    assert_eq!(
        names(vault.matches_for_url("https://bank.test/login")),
        ["Bank"]
    );
    assert!(vault.matches_for_url("https://bank.test/").is_empty());
    assert!(vault.matches_for_url("not a url").is_empty());
    assert_eq!(names(vault.search(" BANK ")), ["Bank"]);
    assert_eq!(names(vault.search("mail-user")), ["Mail"]);
}

#[test]
fn vault_is_locked_on_open_unlocks_offline_and_lock_forgets_everything() {
    let folder = tempfile::tempdir().unwrap();
    snapshot(vec![sealed_login("mail", "Mail", &[("example.com", None)])])
        .save(folder.path())
        .unwrap();

    let mut vault = Vault::open(folder.path()).unwrap();
    assert_eq!(vault.status(), Status::Locked);
    assert_eq!(vault.email(), Some(EMAIL));
    assert!(vault.logins().is_empty());
    assert!(matches!(
        vault.unlock("wrong password"),
        Err(Error::WrongPassword)
    ));
    assert_eq!(vault.status(), Status::Locked);

    vault.unlock(PASSWORD).unwrap();
    assert_eq!(vault.status(), Status::Unlocked);
    let login = &vault.logins()[0];
    assert_eq!(login.name, "Mail");
    assert_eq!(login.username.as_deref(), Some("mail-user"));
    assert_eq!(login.password.as_deref(), Some("mail-secret"));

    vault.lock();
    assert_eq!(vault.status(), Status::Locked);
    assert!(vault.logins().is_empty());
    assert!(vault.matches_for_url("https://example.com/").is_empty());
    assert!(matches!(vault.sync(), Err(Error::Locked)));

    vault.sign_out().unwrap();
    assert_eq!(
        Vault::open(folder.path()).unwrap().status(),
        Status::SignedOut
    );
}

#[test]
fn item_with_its_own_key_decrypts_through_the_user_key() {
    let item_key_bytes: Vec<u8> = (100..164).collect();
    let item_key = SymmetricKey::from_bytes(&item_key_bytes).unwrap();
    let mut login = sealed_login("mail", "ignored", &[]);
    login.key = Some(CipherString::encrypt(&item_key_bytes, &user_key()).unwrap());
    login.name = seal("Keyed", &item_key);
    login.login.totp = Some(seal("JBSWY3DPEHPK3PXP", &item_key));
    login.login.username = None;
    login.login.password = None;
    let opened = login.open(&user_key()).unwrap();
    assert_eq!(opened.name, "Keyed");
    assert_eq!(opened.totp.as_deref(), Some("JBSWY3DPEHPK3PXP"));
    assert_eq!(opened.username, None);
}

struct Request {
    head: String,
    body: String,
}

fn read_request(stream: &mut std::net::TcpStream) -> Request {
    let mut reader = BufReader::new(stream);
    let mut head = String::new();
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        head.push_str(&line);
        if line == "\r\n" {
            break;
        }
    }
    let length = head
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("content-length")
                .then(|| value.trim().parse::<usize>().unwrap())
        })
        .unwrap_or(0);
    let mut body = vec![0; length];
    reader.read_exact(&mut body).unwrap();
    Request {
        head,
        body: String::from_utf8(body).unwrap(),
    }
}

fn sync_body() -> String {
    let login =
        serde_json::to_value(sealed_login("mail", "Mail", &[("example.com", None)])).unwrap();
    let mut organization = login.clone();
    organization["organizationId"] = json!("org-1");
    let mut deleted = login.clone();
    deleted["deletedDate"] = json!("2026-01-01T00:00:00Z");
    let note = json!({ "id": "note", "type": 2, "name": seal("Note", &user_key()), "login": null });
    json!({
        "profile": { "id": "user-1", "email": EMAIL, "key": protected_key() },
        "ciphers": [login, organization, deleted, note],
    })
    .to_string()
}

fn created(body: &str) -> String {
    let mut cipher: serde_json::Value = serde_json::from_str(body).unwrap();
    cipher["id"] = json!("created-1");
    cipher.to_string()
}

fn reply(request: &Request) -> (u16, String) {
    let path = request.head.split_whitespace().nth(1).unwrap();
    match path {
        "/identity/accounts/prelogin" => (200, json!({ "kdf": 0, "kdfIterations": 5000 }).to_string()),
        "/identity/connect/token" if request.body.contains("twoFactorToken=123456") => (
            200,
            json!({ "access_token": "access-1", "expires_in": 3600, "refresh_token": "refresh-1" })
                .to_string(),
        ),
        "/identity/connect/token" if request.body.contains("refresh_token=refresh-token") => (
            200,
            json!({ "access_token": "access-1", "expires_in": 3600, "refresh_token": "refresh-2" })
                .to_string(),
        ),
        "/identity/connect/token" => (
            400,
            json!({ "error": "invalid_grant", "error_description": "Two factor required.", "TwoFactorProviders2": { "0": null, "7": null } })
                .to_string(),
        ),
        "/api/sync?excludeDomains=true" if request.head.contains("Bearer access-1") => (200, sync_body()),
        "/api/ciphers" if request.head.contains("Bearer access-1") => (200, created(&request.body)),
        _ => (404, "{}".to_owned()),
    }
}

fn serve() -> (String, Arc<Mutex<Vec<Request>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = format!("http://{}", listener.local_addr().unwrap());
    let requests = Arc::new(Mutex::new(Vec::new()));
    let log = requests.clone();
    thread::spawn(move || {
        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            let request = read_request(&mut stream);
            let (status, body) = reply(&request);
            let response = format!(
                "HTTP/1.1 {status} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            log.lock().unwrap().push(request);
            stream.write_all(response.as_bytes()).unwrap();
        }
    });
    (address, requests)
}

#[test]
fn sign_in_answers_two_step_login_then_syncs_personal_logins_and_reopens_locked() {
    let (address, requests) = serve();
    let folder = tempfile::tempdir().unwrap();
    let mut vault = Vault::open(folder.path()).unwrap();
    let credentials = Credentials {
        server: Server::parse(&address).unwrap(),
        email: "User@Example.com",
        password: PASSWORD,
    };

    let SignIn::Challenged(challenge) = vault.sign_in(&credentials).unwrap() else {
        panic!("expected a two-step challenge");
    };
    assert_eq!(challenge.methods(), [Method::Authenticator]);
    let wrong = Proof {
        method: Method::Authenticator,
        code: Zeroizing::new("000000".to_owned()),
    };
    assert!(matches!(
        vault.answer(&challenge, &wrong).unwrap(),
        SignIn::Challenged(_)
    ));
    let right = Proof {
        method: Method::Authenticator,
        code: Zeroizing::new(" 123456 ".to_owned()),
    };
    assert!(matches!(
        vault.answer(&challenge, &right).unwrap(),
        SignIn::Unlocked
    ));

    assert_eq!(names(vault.logins().iter().collect()), ["Mail"]);
    let token_request = requests
        .lock()
        .unwrap()
        .iter()
        .find(|request| request.head.starts_with("POST /identity/connect/token"))
        .map(|request| request.body.clone())
        .unwrap();
    assert!(token_request.contains("username=user%40example.com"));
    assert!(token_request.contains("password=0FMeontUyfpu9Ga%2FDvERL9LMAXg9KB82VK6UqHdnKko%3D"));

    let stored: String = std::fs::read_dir(folder.path())
        .unwrap()
        .map(|entry| std::fs::read_to_string(entry.unwrap().path()).unwrap())
        .collect();
    for secret in [PASSWORD, "mail-secret", "refresh-1", "access-1"] {
        assert!(!stored.contains(secret), "{secret} stored in plain text");
    }
    let mut reopened = Vault::open(folder.path()).unwrap();
    assert_eq!(reopened.status(), Status::Locked);
    reopened.unlock(PASSWORD).unwrap();
    assert_eq!(names(reopened.logins().iter().collect()), ["Mail"]);
}

fn vault_on(address: &str, logins: Vec<Sealed>) -> (tempfile::TempDir, Vault) {
    let folder = tempfile::tempdir().unwrap();
    let mut stored = snapshot(logins);
    stored.server = Server::parse(address).unwrap();
    stored.save(folder.path()).unwrap();
    let mut vault = Vault::open(folder.path()).unwrap();
    vault.unlock(PASSWORD).unwrap();
    (folder, vault)
}

fn posted(requests: &Mutex<Vec<Request>>, start: &str) -> Vec<String> {
    requests
        .lock()
        .unwrap()
        .iter()
        .filter(|request| request.head.starts_with(start))
        .map(|request| request.body.clone())
        .collect()
}

#[test]
fn create_login_posts_an_encrypted_item_and_keeps_it_for_offline_unlock() {
    let (address, requests) = serve();
    let (folder, mut vault) = vault_on(&address, Vec::new());

    vault
        .create_login(&NewLogin {
            name: " Forum ",
            username: "forum-user",
            password: "forum-secret",
            uri: "https://forum.example.com/login",
        })
        .unwrap();

    let login = &vault.logins()[0];
    assert_eq!(login.id, CipherId("created-1".to_owned()));
    assert_eq!(login.name, "Forum");
    assert_eq!(login.username.as_deref(), Some("forum-user"));
    assert_eq!(login.password.as_deref(), Some("forum-secret"));
    assert_eq!(login.uris[0].uri, "https://forum.example.com/login");
    assert_eq!(login.uris[0].rule, MatchRule::Domain);

    let [body] = &posted(&requests, "POST /api/ciphers")[..] else {
        panic!("expected one cipher request");
    };
    for secret in ["Forum", "forum-user", "forum-secret", "forum.example.com"] {
        assert!(!body.contains(secret), "{secret} sent in plain text");
    }
    let cipher: serde_json::Value = serde_json::from_str(body).unwrap();
    assert_eq!(cipher["type"], 1);
    let name: CipherString = cipher["name"].as_str().unwrap().parse().unwrap();
    assert_eq!(name.decrypt_text(&user_key()).unwrap(), "Forum");

    let mut reopened = Vault::open(folder.path()).unwrap();
    reopened.unlock(PASSWORD).unwrap();
    assert_eq!(names(reopened.logins().iter().collect()), ["Forum"]);
}

#[test]
fn background_pull_refreshes_the_token_once_and_absorbs_the_server_logins() {
    let (address, requests) = serve();
    let (folder, mut vault) = vault_on(&address, Vec::new());

    let fetched = vault.pull().unwrap().fetch().unwrap();
    assert!(vault.logins().is_empty());
    vault.absorb(fetched).unwrap();
    assert_eq!(names(vault.logins().iter().collect()), ["Mail"]);
    vault.sync().unwrap();
    assert_eq!(posted(&requests, "POST /identity/connect/token").len(), 1);

    let stored = Snapshot::load(folder.path()).unwrap().unwrap();
    assert_eq!(
        stored.refresh_token.decrypt_text(&user_key()).unwrap(),
        "refresh-2"
    );
    vault.lock();
    assert!(matches!(vault.pull(), Err(Error::Locked)));
}
