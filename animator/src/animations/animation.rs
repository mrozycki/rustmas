use rustmas_light_client as client;

pub trait Animation {
    fn frame(&self, time: f64) -> client::Frame;
}
