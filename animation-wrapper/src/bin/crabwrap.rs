use std::{
    fs::{read_dir, File},
    io::{BufReader, Read},
    path::PathBuf,
    process::{Command, ExitStatus},
};

fn find_project_name() -> std::io::Result<String> {
    #[derive(serde::Deserialize)]
    struct Package {
        name: String,
    }

    #[derive(serde::Deserialize)]
    struct CargoToml {
        package: Package,
    }

    let cargo_toml_path = std::env::current_dir()?.join("Cargo.toml");
    let mut buf = Vec::new();
    BufReader::new(File::open(&cargo_toml_path)?).read_to_end(&mut buf)?;
    let cargo_toml = toml::from_str::<CargoToml>(&String::from_utf8(buf).map_err(|_| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Cargo.toml is not valid UTF-8",
        )
    })?)
    .map_err(|_| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Cargo.toml is not valid TOML",
        )
    })?;

    Ok(cargo_toml.package.name)
}

fn build_plugin() -> std::io::Result<ExitStatus> {
    let mut cmd = Command::new("cargo")
        .args(["build", "--target", "wasm32-wasip2", "--release"])
        .spawn()?;
    cmd.wait()
}

fn find_animation_id() -> std::io::Result<String> {
    std::env::current_dir()?
        .components()
        .next_back()
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
    let project_name = find_project_name()?.replace('-', "_");

    let executable_path = project_root
        .join("target/wasm32-wasip2/release")
        .join(format!("{project_name}.wasm"));

    if !executable_path.exists() {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Could not find animation executable at {executable_path:?}"),
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

fn main() -> std::io::Result<()> {
    let manifest_path = find_manifest()?;

    build_plugin()?;

    let executable_path = find_animation_executable()?;
    let output_path = get_output_path()?;

    animation_wrapper::wrap::wrap_plugin(&output_path, &executable_path, &manifest_path)?;

    println!("Plugin ready at {}", output_path.to_string_lossy());

    Ok(())
}
