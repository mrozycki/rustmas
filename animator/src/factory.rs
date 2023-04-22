use std::path::{Path, PathBuf};

use log::info;

use crate::jsonrpc_animation::{AnimationPlugin, JsonRpcEndpoint};

pub struct AnimationFactory {
    plugin_dir: PathBuf,
    points: Vec<(f64, f64, f64)>,
}

impl AnimationFactory {
    pub fn new<P: AsRef<Path>>(plugin_dir: P, points: Vec<(f64, f64, f64)>) -> Self {
        Self {
            plugin_dir: plugin_dir.as_ref().to_owned(),
            points,
        }
    }

    #[cfg(windows)]
    fn make_path(&self, name: &str) -> PathBuf {
        self.plugin_dir.join(Path::new(name).with_extension("exe"))
    }

    #[cfg(not(windows))]
    fn make_path(&self, name: &str) -> PathBuf {
        self.plugin_dir.join(name)
    }

    pub fn make(&self, name: &str) -> AnimationPlugin {
        info!(
            "Trying to start plugin app {} from directory {:?}",
            name, self.plugin_dir
        );

        AnimationPlugin::new(
            JsonRpcEndpoint::new(self.make_path(name)).unwrap(),
            self.points.clone(),
        )
    }
}
