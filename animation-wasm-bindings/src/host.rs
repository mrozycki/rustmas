use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

use animation_api::{event::Event, schema};
use exports::guest::animation::plugin::{Color, Position};
use itertools::Itertools;
use tokio::sync::Mutex;
use wasmtime::{
    AsContextMut, Config, Engine, Store,
    component::{Component, Linker, ResourceAny, bindgen},
};
use wasmtime_wasi::{
    ResourceTable,
    p2::{IoView, WasiCtx, WasiCtxBuilder, WasiView},
};

bindgen!({
    world: "animation",
    async: true,
});

struct State {
    ctx: WasiCtx,
    table: ResourceTable,
}

impl WasiView for State {
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.ctx
    }
}
impl IoView for State {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
}

#[derive(Debug, thiserror::Error)]
pub enum HostedPluginError {
    #[error("wasmtime returned error: {0}")]
    WasmtimeError(#[from] wasmtime::Error),

    #[error("cannot open plugin: {0}")]
    PluginOpenError(#[from] std::io::Error),
}
type Result<T> = std::result::Result<T, HostedPluginError>;

pub struct HostedPlugin {
    store: Mutex<Store<State>>,
    bindings: Animation,
    handle: ResourceAny,
}

impl HostedPlugin {
    pub async fn new(executable_path: &Path, points: Vec<(f64, f64, f64)>) -> Result<Self> {
        let reader = BufReader::new(File::open(executable_path)?);
        Self::from_reader(reader, points).await
    }

    pub async fn from_reader<R: Read>(mut reader: R, points: Vec<(f64, f64, f64)>) -> Result<Self> {
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;

        let mut config = Config::new();
        config.async_support(true);
        let engine = Engine::new(&config)?;
        let component = Component::from_binary(&engine, &data)?;

        let mut linker = Linker::new(&engine);
        wasmtime_wasi::p2::add_to_linker_async(&mut linker)?;

        let mut store = Store::new(
            &engine,
            State {
                ctx: WasiCtxBuilder::new().build(),
                table: ResourceTable::new(),
            },
        );

        let bindings = Animation::instantiate_async(&mut store, &component, &linker)
            .await
            .unwrap();
        let guest = bindings.guest_animation_plugin();
        let animation = guest.animation();
        let points = points
            .into_iter()
            .map(|(x, y, z)| Position { x, y, z })
            .collect_vec();
        let handle = animation
            .call_constructor(&mut store, &points)
            .await
            .unwrap();

        Ok(Self {
            store: Mutex::new(store),
            bindings,
            handle,
        })
    }

    pub async fn update(&self, time_delta: f64) -> Result<()> {
        let mut store = self.store.lock().await;
        self.bindings
            .guest_animation_plugin()
            .animation()
            .call_update(store.as_context_mut(), self.handle, time_delta)
            .await
            .map_err(Into::into)
    }

    pub async fn render(&self) -> Result<Vec<Color>> {
        let mut store = self.store.lock().await;
        self.bindings
            .guest_animation_plugin()
            .animation()
            .call_render(store.as_context_mut(), self.handle)
            .await
            .map_err(Into::into)
    }

    pub async fn get_schema(&self) -> Result<schema::ConfigurationSchema> {
        let mut store = self.store.lock().await;
        let schema = self
            .bindings
            .guest_animation_plugin()
            .animation()
            .call_get_schema(store.as_context_mut(), self.handle)
            .await?;

        Ok(serde_json::from_str(&schema).unwrap())
    }

    pub async fn set_parameters(
        &mut self,
        values: &HashMap<String, schema::ParameterValue>,
    ) -> Result<()> {
        let mut store = self.store.lock().await;
        let values = serde_json::to_string(values).unwrap();

        self.bindings
            .guest_animation_plugin()
            .animation()
            .call_set_parameters(store.as_context_mut(), self.handle, &values)
            .await
            .map_err(Into::into)
    }

    pub async fn get_parameters(&self) -> Result<HashMap<String, schema::ParameterValue>> {
        let mut store = self.store.lock().await;
        let values = self
            .bindings
            .guest_animation_plugin()
            .animation()
            .call_get_parameters(store.as_context_mut(), self.handle)
            .await?;

        Ok(serde_json::from_str(&values).unwrap_or_default())
    }

    pub async fn get_fps(&self) -> Result<f64> {
        let mut store = self.store.lock().await;
        self.bindings
            .guest_animation_plugin()
            .animation()
            .call_get_fps(store.as_context_mut(), self.handle)
            .await
            .map_err(Into::into)
    }

    pub async fn send_event(&self, event: Event) -> Result<()> {
        let mut store = self.store.lock().await;
        let event = serde_json::to_string(&event).unwrap();

        self.bindings
            .guest_animation_plugin()
            .animation()
            .call_on_event(store.as_context_mut(), self.handle, &event)
            .await
            .map_err(Into::into)
    }
}
