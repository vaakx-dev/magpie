mod address;
mod focus;
mod java;
mod sponsorblock;
mod update;
mod vault;

use std::{
    path::{Path, PathBuf},
    ptr::null_mut,
};

use jni::{
    JNIEnv,
    objects::{JClass, JString},
    sys::{jboolean, jbyteArray, jstring},
};
use magpie_engine::{bitwarden::password, blocker::snapshot, sponsorblock::Bridge};

const FILTERS_FOLDER: &str = "filters";
const BITWARDEN_FOLDER: &str = "bitwarden";

fn filters(
    env: &mut JNIEnv,
    data_dir: &JString,
    snapshot: fn(&Path) -> Option<Vec<u8>>,
) -> jbyteArray {
    java::text(env, data_dir)
        .and_then(|data_dir| snapshot(&PathBuf::from(data_dir).join(FILTERS_FOLDER)))
        .map_or(null_mut(), |snapshot| java::bytes(env, &snapshot))
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_app_magpie_browser_core_MagpieCore_cleanUrl<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    url: JString<'local>,
    strip_tracking: jboolean,
) -> jstring {
    let Some(url) = java::text(&mut env, &url) else {
        return null_mut();
    };
    let clean = match java::truth(strip_tracking) {
        true => address::without_trackers(&url),
        false => url,
    };
    java::string(&mut env, &clean)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_app_magpie_browser_core_MagpieCore_focusScript<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    blocked: JString<'local>,
) -> jstring {
    let Some(blocked) = java::text(&mut env, &blocked) else {
        return null_mut();
    };
    java::string(&mut env, &focus::rules(&blocked).script())
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_app_magpie_browser_core_MagpieCore_focusUpdate<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    blocked: JString<'local>,
) -> jstring {
    let Some(blocked) = java::text(&mut env, &blocked) else {
        return null_mut();
    };
    java::string(&mut env, &focus::rules(&blocked).update())
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_app_magpie_browser_core_MagpieCore_focusRedirect<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    blocked: JString<'local>,
    url: JString<'local>,
) -> jstring {
    let (Some(blocked), Some(url)) = (java::text(&mut env, &blocked), java::text(&mut env, &url))
    else {
        return null_mut();
    };
    focus::redirect(&focus::rules(&blocked), &url)
        .map_or(null_mut(), |watch| java::string(&mut env, &watch))
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_app_magpie_browser_core_MagpieCore_sponsorBlockScript<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
) -> jstring {
    java::string(&mut env, Bridge::GeckoView.script())
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_app_magpie_browser_core_MagpieCore_sponsorBlockView<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    config: JString<'local>,
) -> jstring {
    let Some(config) = java::text(&mut env, &config) else {
        return null_mut();
    };
    java::string(&mut env, &sponsorblock::view(&config))
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_app_magpie_browser_core_MagpieCore_sponsorBlockSwitch<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    config: JString<'local>,
    key: JString<'local>,
    on: jboolean,
) -> jstring {
    let (Some(config), Some(key)) = (java::text(&mut env, &config), java::text(&mut env, &key))
    else {
        return null_mut();
    };
    java::string(
        &mut env,
        &sponsorblock::switched(&config, &key, java::truth(on)),
    )
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_app_magpie_browser_core_MagpieCore_sponsorBlockResume<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    config: JString<'local>,
    channel: JString<'local>,
) -> jstring {
    let (Some(config), Some(channel)) = (
        java::text(&mut env, &config),
        java::text(&mut env, &channel),
    ) else {
        return null_mut();
    };
    java::string(&mut env, &sponsorblock::resumed(&config, &channel))
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_app_magpie_browser_core_MagpieCore_sponsorBlockAnswer<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    config: JString<'local>,
    request: JString<'local>,
) -> jstring {
    let (Some(config), Some(request)) = (
        java::text(&mut env, &config),
        java::text(&mut env, &request),
    ) else {
        return null_mut();
    };
    match sponsorblock::answer(&config, &request) {
        Some(reply) => java::string(&mut env, &reply),
        None => null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_app_magpie_browser_core_MagpieCore_cachedFilters<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    data_dir: JString<'local>,
) -> jbyteArray {
    filters(&mut env, &data_dir, snapshot::cached)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_app_magpie_browser_core_MagpieCore_refreshedFilters<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    data_dir: JString<'local>,
) -> jbyteArray {
    filters(&mut env, &data_dir, snapshot::refreshed)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_app_magpie_browser_core_MagpieCore_init<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    data_dir: JString<'local>,
) {
    let Some(data_dir) = java::text(&mut env, &data_dir) else {
        return;
    };
    vault::locate(PathBuf::from(data_dir).join(BITWARDEN_FOLDER));
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_app_magpie_browser_core_MagpieCore_bitwardenStatus<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
) -> jstring {
    java::string(&mut env, &vault::status())
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_app_magpie_browser_core_MagpieCore_bitwardenSignIn<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    server: JString<'local>,
    email: JString<'local>,
    password: JString<'local>,
) -> jstring {
    let (Some(server), Some(email), Some(password)) = (
        java::text(&mut env, &server),
        java::text(&mut env, &email),
        java::text(&mut env, &password),
    ) else {
        return null_mut();
    };
    java::string(&mut env, &vault::sign_in(&server, &email, &password))
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_app_magpie_browser_core_MagpieCore_bitwardenAnswer<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    method: JString<'local>,
    code: JString<'local>,
) -> jstring {
    let (Some(method), Some(code)) = (java::text(&mut env, &method), java::text(&mut env, &code))
    else {
        return null_mut();
    };
    java::string(&mut env, &vault::answer(&method, &code))
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_app_magpie_browser_core_MagpieCore_bitwardenSendEmailCode<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
) -> jstring {
    java::string(&mut env, &vault::send_email_code())
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_app_magpie_browser_core_MagpieCore_bitwardenUnlock<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    password: JString<'local>,
) -> jstring {
    let Some(password) = java::text(&mut env, &password) else {
        return null_mut();
    };
    java::string(&mut env, &vault::unlock(&password))
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_app_magpie_browser_core_MagpieCore_bitwardenLock<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
) -> jstring {
    java::string(&mut env, &vault::lock())
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_app_magpie_browser_core_MagpieCore_bitwardenMatches<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    url: JString<'local>,
) -> jstring {
    let Some(url) = java::text(&mut env, &url) else {
        return null_mut();
    };
    java::string(&mut env, &vault::matches(&url))
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_app_magpie_browser_core_MagpieCore_bitwardenSearch<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    query: JString<'local>,
) -> jstring {
    let Some(query) = java::text(&mut env, &query) else {
        return null_mut();
    };
    java::string(&mut env, &vault::search(&query))
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_app_magpie_browser_core_MagpieCore_bitwardenTotp<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    secret: JString<'local>,
) -> jstring {
    let Some(secret) = java::text(&mut env, &secret) else {
        return null_mut();
    };
    java::string(&mut env, &vault::totp(&secret))
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_app_magpie_browser_core_MagpieCore_bitwardenGeneratePassword<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
) -> jstring {
    match password::generate() {
        Ok(password) => java::string(&mut env, &password),
        Err(error) => java::throw(&mut env, &error.to_string()),
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_app_magpie_browser_core_MagpieCore_updateDue<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    data_dir: JString<'local>,
) -> jboolean {
    let due =
        java::text(&mut env, &data_dir).is_some_and(|data_dir| update::due(Path::new(&data_dir)));
    java::boolean(due)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_app_magpie_browser_core_MagpieCore_updateCheck<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    data_dir: JString<'local>,
    version: JString<'local>,
    abis: JString<'local>,
    channel: JString<'local>,
) -> jstring {
    let (Some(data_dir), Some(version), Some(abis), Some(channel)) = (
        java::text(&mut env, &data_dir),
        java::text(&mut env, &version),
        java::text(&mut env, &abis),
        java::text(&mut env, &channel),
    ) else {
        return null_mut();
    };
    java::string(
        &mut env,
        &update::latest(Path::new(&data_dir), &version, &abis, &channel),
    )
}
