use std::{
    collections::HashMap,
    error::Error,
    path::{Path, PathBuf},
};

use log::info;
use serde::Deserialize;

use crate::jsonrpc_animation::{AnimationPlugin, JsonRpcEndpoint, JsonRpcEndpointError};

#[derive(Deserialize)]
pub struct PluginManifest {
    pub display_name: String,
}

pub struct Plugin {
    pub manifest: PluginManifest,
    pub path: PathBuf,
}

impl Plugin {
    fn start(&self) -> Result<JsonRpcEndpoint, JsonRpcEndpointError> {
        #[cfg(windows)]
        let executable_name = "plugin.exe";

        #[cfg(not(windows))]
        let executable_name = "plugin";

        JsonRpcEndpoint::new(self.path.join(executable_name))
    }
}

pub struct AnimationFactory {
    plugin_dir: PathBuf,
    plugins: HashMap<String, Plugin>,
    points: Vec<(f64, f64, f64)>,
}

impl AnimationFactory {
    pub fn new<P: AsRef<Path>>(plugin_dir: P, points: Vec<(f64, f64, f64)>) -> Self {
        Self {
            plugin_dir: plugin_dir.as_ref().to_owned(),
            plugins: HashMap::new(),
            points,
        }
    }

    pub fn discover(&mut self) -> Result<(), Box<dyn Error>> {
        self.plugins = glob::glob(&format!(
            "{}/*/manifest.json",
            self.plugin_dir
                .to_str()
                .ok_or("Plugins directory path is not valid UTF-8")?
        ))?
        .map(|path| {
            path.map_err(|e| -> Box<dyn Error> { Box::new(e) })
                .and_then(|path| -> Result<_, Box<dyn Error>> {
                    let manifest: PluginManifest = serde_json::from_slice(&std::fs::read(&path)?)?;
                    let path = path.parent().unwrap().to_owned();
                    let id = path
                        .file_name()
                        .unwrap()
                        .to_str()
                        .ok_or("Plugin name not valid UTF-8")?
                        .to_owned();
                    Ok((id, Plugin { path, manifest }))
                })
        })
        .collect::<Result<_, _>>()?;

        Ok(())
    }

    pub fn list(&self) -> &HashMap<String, Plugin> {
        &self.plugins
    }

    pub fn make(&self, name: &str) -> Result<AnimationPlugin, Box<dyn Error>> {
        let Some(plugin) = self.plugins.get(name) else {
            return Err(format!("No plugin with name '{}'", name).into());
        };

        info!(
            "Trying to start plugin app '{}' from directory '{:?}'",
            name, plugin.path
        );

        Ok(AnimationPlugin::new(plugin.start()?, self.points.clone()))
    }
}
