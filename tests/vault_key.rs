use std::path::Path;

use magpie::vault_key::VaultKey;
use magpie_engine::bitwarden::{CipherString, Kdf, Server, Snapshot, Status, SymmetricKey, Vault};
use tempfile::TempDir;

const PASSWORD: &str = "correct horse battery staple";
const EMAIL: &str = "user@example.com";
const KDF: Kdf = Kdf::Pbkdf2 { iterations: 5000 };

const USER_KEY: [u8; 64] = [7; 64];
const ROTATED_USER_KEY: [u8; 64] = [9; 64];

fn signed_in(folder: &Path, user_key_bytes: &[u8]) {
    let user_key = SymmetricKey::from_bytes(user_key_bytes).unwrap();
    let master_key = KDF.master_key(PASSWORD, EMAIL).unwrap().stretch();
    Snapshot {
        server: Server::Europe,
        email: EMAIL.to_owned(),
        kdf: KDF,
        protected_key: CipherString::encrypt(user_key_bytes, &master_key).unwrap(),
        refresh_token: CipherString::encrypt(b"refresh-token", &user_key).unwrap(),
        logins: Vec::new(),
    }
    .save(folder)
    .unwrap();
}

fn unlocked_with_password() -> (TempDir, Vault, VaultKey) {
    let folder = tempfile::tempdir().unwrap();
    signed_in(folder.path(), &USER_KEY);
    let mut vault = Vault::open(folder.path()).unwrap();
    vault.unlock(PASSWORD).unwrap();
    let key = VaultKey::in_folder(folder.path());
    key.save(&vault).unwrap();
    (folder, vault, key)
}

fn relaunched(folder: &TempDir) -> Vault {
    let mut vault = Vault::open(folder.path()).unwrap();
    assert_eq!(vault.status(), Status::Locked);
    VaultKey::in_folder(folder.path()).unlock(&mut vault).ok();
    vault
}

#[test]
fn saved_key_unlocks_the_vault_on_relaunch_without_the_master_password() {
    let (folder, vault, _) = unlocked_with_password();
    drop(vault);

    assert_eq!(relaunched(&folder).status(), Status::Unlocked);
}

#[test]
fn lock_keeps_the_vault_locked_on_relaunch_until_the_master_password() {
    let (folder, mut vault, key) = unlocked_with_password();
    key.lock(&mut vault).unwrap();
    assert_eq!(vault.status(), Status::Locked);
    drop(vault);

    let mut vault = relaunched(&folder);
    assert_eq!(vault.status(), Status::Locked);
    vault.unlock(PASSWORD).unwrap();
    assert_eq!(vault.status(), Status::Unlocked);
}

#[test]
fn sign_out_wipes_the_saved_key_so_a_new_sign_in_starts_locked() {
    let (folder, mut vault, key) = unlocked_with_password();
    key.sign_out(&mut vault).unwrap();
    assert_eq!(vault.status(), Status::SignedOut);
    drop(vault);

    signed_in(folder.path(), &USER_KEY);
    assert_eq!(relaunched(&folder).status(), Status::Locked);
}

#[test]
fn stale_saved_key_falls_back_to_the_master_password() {
    let (folder, vault, _) = unlocked_with_password();
    drop(vault);
    signed_in(folder.path(), &ROTATED_USER_KEY);

    let mut vault = relaunched(&folder);
    assert_eq!(vault.status(), Status::Locked);
    vault.unlock(PASSWORD).unwrap();
    assert_eq!(vault.status(), Status::Unlocked);
}
