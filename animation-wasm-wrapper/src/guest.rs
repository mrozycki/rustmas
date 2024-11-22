use std::{cell::RefCell, marker::PhantomData};

use animation_api::schema::GetSchema;
use exports::guest::animation::plugin::{Color, Guest, Position};

wit_bindgen::generate!({
    world: "animation",
    pub_export_macro: true,
});

pub struct GuestPluginWrapper<T: animation_api::Animation> {
    _phantom: PhantomData<T>,
}

impl<T: animation_api::Animation + 'static> Guest for GuestPluginWrapper<T> {
    type Animation = GuestAnimationWrapper<<T as animation_api::Animation>::Wrapped>;
}

pub struct GuestAnimationWrapper<T: animation_api::Animation<Parameters: GetSchema>> {
    inner: RefCell<T>,
}

impl<T: animation_api::Animation + 'static> exports::guest::animation::plugin::GuestAnimation
    for GuestAnimationWrapper<T>
{
    fn new(points: Vec<Position>) -> Self {
        Self {
            inner: RefCell::new(T::new(
                points.into_iter().map(|p| (p.x, p.y, p.z)).collect(),
            )),
        }
    }

    fn update(&self, time_delta: f64) {
        self.inner.borrow_mut().update(time_delta);
    }

    fn render(&self) -> Vec<Color> {
        self.inner
            .borrow()
            .render()
            .pixels_iter()
            .map(|p| Color {
                r: p.r,
                g: p.g,
                b: p.b,
            })
            .collect()
    }

    fn get_schema(&self) -> String {
        serde_json::to_string(&self.inner.borrow().get_schema()).unwrap()
    }

    fn get_parameters(&self) -> String {
        serde_json::to_string(&self.inner.borrow().get_parameters()).unwrap()
    }

    fn set_parameters(&self, values: String) {
        if let Ok(values) = serde_json::from_str(&values) {
            self.inner.borrow_mut().set_parameters(values);
        }
    }

    fn get_fps(&self) -> f64 {
        self.inner.borrow().get_fps()
    }

    fn on_event(&self, event: String) {
        if let Ok(event) = serde_json::from_str(&event) {
            self.inner.borrow_mut().on_event(event);
        }
    }
}
