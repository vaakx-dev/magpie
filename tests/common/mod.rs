use std::process::Command;

pub fn node_accepts(script: &[u8]) -> bool {
    let folder = tempfile::tempdir().unwrap();
    let file = folder.path().join("page.js");
    std::fs::write(&file, script).unwrap();
    let status = Command::new("node").arg("--check").arg(&file).status();
    status
        .expect("node must be installed to check page scripts")
        .success()
}
