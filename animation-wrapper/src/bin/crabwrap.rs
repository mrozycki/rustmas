use std::{
    fs::read_dir,
    path::PathBuf,
    process::{Command, ExitStatus},
};

fn build_plugin() -> std::io::Result<ExitStatus> {
    let mut cmd = Command::new("cargo")
        .args(["build", "--target", "wasm32-wasip2", "--release"])
        .spawn()?;
    cmd.wait()
}

fn find_animation_id() -> std::io::Result<String> {
    std::env::current_dir()?
        .components()
        .last()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Invalid project path"))
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .map(|s| s.replace('-', "_"))
}

fn find_manifest() -> std::io::Result<PathBuf> {
    let manifest_path = std::env::current_dir()?.join("manifest.json");

    if !manifest_path.exists() {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not find manifest.json",
        ))
    } else {
        Ok(manifest_path)
    }
}

fn find_animation_executable() -> std::io::Result<PathBuf> {
    let project_root = find_project_root()?;
    let animation_id = find_animation_id()?;

    let executable_path = project_root
        .join("target/wasm32-wasip2/release")
        .join(format!("{animation_id}.wasm"));

    if !executable_path.exists() {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not find animation executable",
        ))
    } else {
        Ok(executable_path)
    }
}

fn get_output_path() -> std::io::Result<PathBuf> {
    let animation_id = find_animation_id()?;
    Ok(std::env::current_dir()?
        .as_path()
        .join(format!("{animation_id}.crab")))
}

fn find_project_root() -> std::io::Result<PathBuf> {
    std::env::current_dir()?
        .as_path()
        .ancestors()
        .find(|p| {
            let Ok(dir) = read_dir(p) else {
                return false;
            };
            dir.into_iter()
                .filter_map(|p| p.ok())
                .map(|p| p.file_name().to_string_lossy().to_string())
                .any(|p| p == "Cargo.lock")
        })
        .map(PathBuf::from)
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Could not find Cargo.lock until filesystem root",
            )
        })
}

fn main() {
    let manifest_path = find_manifest().expect("Could not find manifest.json file");

    build_plugin().expect("Failed to build the plugin");

    let executable_path = find_animation_executable().expect("Could not find animation executable");
    let output_path = get_output_path().expect("Could not generate output path");

    animation_wrapper::wrap::wrap_plugin(&output_path, &executable_path, &manifest_path)
        .expect("Failed to wrap the plugin");

    println!("Plugin ready at {}", output_path.to_string_lossy());
}
