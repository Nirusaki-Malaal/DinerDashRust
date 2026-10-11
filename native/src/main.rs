#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
fn main() {
    diner_dash::run();
    #[cfg(not(target_os = "android"))]
    std::process::exit(0);
}
