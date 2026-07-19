fn main() {
    println!("cargo:rerun-if-changed=packaging/windows/Kimini.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    winresource::WindowsResource::new()
        .set_icon("packaging/windows/Kimini.ico")
        .set("ProductName", "Kimini")
        .set("FileDescription", "Kimi Code desktop client")
        .compile()
        .expect("embed the Windows application resources");
}
