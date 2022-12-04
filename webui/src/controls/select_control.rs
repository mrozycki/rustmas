use lightfx::parameter_schema::{Parameter, ParameterValue};
use web_sys::HtmlSelectElement;
use yew::{html, Component, Context, Html, NodeRef};

use super::ParameterControlProps;

#[derive(Default)]
pub struct SelectParameterControl {
    node_ref: NodeRef,
}

impl Component for SelectParameterControl {
    type Message = ();
    type Properties = ParameterControlProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Default::default()
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if let Parameter {
            id,
            value: ParameterValue::Enum { values },
            ..
        } = ctx.props().schema.clone()
        {
            let selected_index = ctx
                .props()
                .value
                .as_ref()
                .and_then(|v| v.as_str())
                .and_then(|v| values.iter().position(|opt| opt.value == v))
                .unwrap_or(0);

            html! {
                <select name={id} ref={self.node_ref.clone()} data-selected-index={selected_index.to_string()}>
                    {values.into_iter().map(|item| {
                        let value = format!("\"{}\"", item.value);
                        html!(
                        <option {value}><strong>{item.name}</strong> {item.description.unwrap_or_default()}</option>
                    )}).collect::<Html>()}
                </select>
            }
        } else {
            html!()
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {
        self.node_ref
            .cast::<HtmlSelectElement>()
            .and_then(|elem| elem.get_attribute("data-selected-index"))
            .and_then(|attr| attr.parse::<i32>().ok())
            .and_then(|index| {
                self.node_ref.cast::<HtmlSelectElement>().and_then(|elem| {
                    elem.set_selected_index(index);
                    Some(())
                })
            });
    }
}
