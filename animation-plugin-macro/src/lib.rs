use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_attribute]
pub fn plugin(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let ast: syn::ItemStruct = syn::parse2(item.clone().into()).unwrap();
    let name = &ast.ident;
    let output = quote! {
        #ast

        fn main() {
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
            let mut animation: Option<#name> = None;
            let mut stdin = BufReader::new(std::io::stdin());

            loop {
                match receive(&mut stdin) {
                    Ok(Some(message)) => match message.payload {
                        JsonRpcMethod::Initialize { points } => {
                            animation = None;
                            animation = Some(<#name>::new(points));
                        }
                        JsonRpcMethod::AnimationName => {
                            if let Some(animation) = animation.as_ref() {
                                respond(message.id, animation.animation_name());
                            }
                        },
                        JsonRpcMethod::ParameterSchema => {
                            if let Some(animation) = animation.as_ref() {
                                respond(message.id, animation.parameter_schema());
                            }
                        },
                        JsonRpcMethod::SetParameters { params } => {
                            if let Some(mut animation) = animation.as_mut() {
                                let _ = animation.set_parameters(params);
                            }
                        },
                        JsonRpcMethod::GetParameters => {
                            if let Some(animation) = animation.as_ref() {
                                respond(message.id, animation.get_parameters());
                            }
                        },
                        JsonRpcMethod::GetFps => {
                            if let Some(animation) = animation.as_ref() {
                                respond(message.id, animation.get_fps());
                            }
                        },
                        JsonRpcMethod::Update { time_delta } => {
                            if let Some(mut animation) = animation.as_mut() {
                                animation.update(time_delta);
                            }
                        },
                        JsonRpcMethod::Render => {
                            if let Some(animation) = animation.as_ref() {
                                respond(message.id, animation.render());
                            }
                        },
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
    };

    output.into()
}
