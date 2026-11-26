fn main() {
    println!("cargo:rerun-if-changed=assets/magpie.rc");
    println!("cargo:rerun-if-changed=assets/magpie.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        embed_resource::compile("assets/magpie.rc", embed_resource::NONE)
            .manifest_optional()
            .expect("the Windows icon resource compiles");
    }
}
