fn main() {
    prepare_runtime();
    spy_hunter::run();
}

/// When running a packaged Linux build, the binary sits next to an `assets/`
/// directory. Move there so Bevy's normal relative asset lookup finds it.
#[cfg(not(target_arch = "wasm32"))]
fn prepare_runtime() {
    if std::env::var_os("BEVY_ASSET_ROOT").is_some() {
        return;
    }
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
        && dir.join("assets").is_dir()
    {
        let _ = std::env::set_current_dir(dir);
    }
}

#[cfg(target_arch = "wasm32")]
fn prepare_runtime() {}
