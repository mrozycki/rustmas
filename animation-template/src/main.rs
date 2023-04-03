mod animation;

use std::{
    error::Error,
    io::{BufRead, BufReader},
};

use animation_api::{JsonRpcMessage, JsonRpcMethod};
use serde::Serialize;
use serde_json::json;

fn receive(
    reader: &mut impl BufRead,
) -> Result<Option<JsonRpcMessage<JsonRpcMethod>>, Box<dyn Error>> {
    let mut buffer = String::new();
    if reader.read_line(&mut buffer)? == 0 {
        Ok(None)
    } else {
        Ok(Some(serde_json::from_str(&buffer)?))
    }
}

fn respond<T>(id: Option<usize>, payload: T)
where
    T: Serialize,
{
    let Some(id) = id else { return; };

    println!(
        "{}",
        json!({
            "id": id,
            "result": payload,
        })
    );
}

fn main() {
    let mut animation = animation::Animation::new(Vec::new());
    let mut stdin = BufReader::new(std::io::stdin());

    loop {
        match receive(&mut stdin) {
            Ok(Some(message)) => match message.payload {
                JsonRpcMethod::Initialize { points } => {
                    animation = animation::Animation::new(points)
                }
                JsonRpcMethod::AnimationName => respond(message.id, animation.animation_name()),
                JsonRpcMethod::ParameterSchema => respond(message.id, animation.parameter_schema()),
                JsonRpcMethod::SetParameters { params } => {
                    let _ = animation.set_parameters(params);
                }
                JsonRpcMethod::GetParameters => respond(message.id, animation.get_parameters()),
                JsonRpcMethod::GetFps => respond(message.id, animation.get_fps()),
                JsonRpcMethod::Update { time_delta } => {
                    animation.update(time_delta);
                }
                JsonRpcMethod::Render => respond(message.id, animation.render()),
            },
            Ok(None) => {
                break;
            }
            Err(err) => {
                eprintln!("Animation error: {:?}", err);
                break;
            }
        }
    }
}
