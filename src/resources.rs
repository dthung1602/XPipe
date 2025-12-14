use std::path::PathBuf;

use log::debug;

fn res_dir() -> anyhow::Result<PathBuf> {
    let current_exe = std::env::current_exe()?;
    Ok(current_exe
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("res"))
}

pub async fn load_string(file_name: &str) -> anyhow::Result<String> {
    debug!("Loading resource: {:?}", file_name);
    let path = res_dir()?.join(file_name);
    Ok(std::fs::read_to_string(path)?)
}
