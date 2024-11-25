use yew::{html, Callback, Html};

use crate::{animations::AnimationList, controls::ParameterControlList, window_width};

#[yew::function_component(AnimationControl)]
pub fn animation_control() -> Html {
    let animation_id = yew::use_state::<Option<String>, _>(|| None);
    let dirty = yew::use_state(|| false);

    let animation_switched_callback = Callback::from({
        let animation_id = animation_id.clone();
        let dirty = dirty.clone();
        move |new_animation_id: Option<String>| {
            animation_id.set(new_animation_id);
            dirty.set(false);
        }
    });

    let parameters_dirty = Callback::from({
        let dirty = dirty.clone();
        move |new_dirty| dirty.set(new_dirty)
    });

    html! {
        <>
            <AnimationList dirty={*dirty} {animation_switched_callback} />
            {
                if window_width() > 640 {
                    html!(<super::Visualizer />)
                } else {
                    html!()
                }
            }
            <ParameterControlList
                animation_id={(*animation_id).clone()}
                parameters_dirty={parameters_dirty} />
        </>
    }
}
