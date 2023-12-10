use yew::{html, prelude::Html, Component, Context};

#[derive(Default)]
pub struct Dummy {}

impl Component for Dummy {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Default::default()
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        false
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html!()
    }
}
