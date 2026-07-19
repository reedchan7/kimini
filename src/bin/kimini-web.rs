#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

fn main() -> wry::Result<()> {
    kimini::legacy_web::run()
}
