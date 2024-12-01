use darling::{FromField, FromMeta, FromVariant};
use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn wasm_plugin(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let ast: syn::ItemStruct = syn::parse2(item.into()).unwrap();
    generate_wasm_plugin(ast).into()
}

fn generate_wasm_plugin(ast: syn::ItemStruct) -> proc_macro2::TokenStream {
    let name = &ast.ident;
    quote! {
        #ast

        type WrappedGuestPlugin = animation_wasm_bindings::guest::GuestPluginBindings<#name>;
        animation_wasm_bindings::guest::export!(WrappedGuestPlugin with_types_in animation_wasm_bindings::guest);
    }
}

#[proc_macro_attribute]
pub fn plugin(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let ast: syn::ItemStruct = syn::parse2(item.into()).unwrap();
    generate_native_plugin(ast).into()
}

fn generate_native_plugin(ast: syn::ItemStruct) -> proc_macro2::TokenStream {
    let name = &ast.ident;
    quote! {
        #ast

        fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
            use std::{
                error::Error,
                io::{BufRead, BufReader},
            };

            use animation_api::{JsonRpcMessage, JsonRpcMethod, JsonRpcError, ErrorType, AnimationError};
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

            fn error(id: Option<usize>, message: String)
            {
                let Some(id) = id else { return; };

                println!(
                    "{}",
                    json!({
                        "id": id,
                        "error": JsonRpcError {
                            code: ErrorType::AnimationError,
                            message: "Animation Error".into(),
                            data: AnimationError {
                                message
                            }
                        },
                    })
                );
            }

            let mut animation = None;
            let mut stdin = BufReader::new(std::io::stdin());

            loop {
                match receive(&mut stdin) {
                    Ok(Some(message)) => match message.payload {
                        JsonRpcMethod::Initialize { points } => {
                            animation = None;
                            animation = Some(<#name>::new_wrapped(points));
                            respond(message.id, ());
                        }
                        JsonRpcMethod::ParameterSchema => {
                            if let Some(animation) = animation.as_ref() {
                                respond(message.id, animation.get_schema());
                            }
                        },
                        JsonRpcMethod::SetParameters { params } => {
                            if let Some(mut animation) = animation.as_mut() {
                                match serde_json::from_value(serde_json::json!(params)) {
                                    Ok(params) => {
                                        animation.set_parameters(params);
                                        respond(message.id, ());
                                    }
                                    Err(e) => error(message.id, e.to_string())
                                }
                            }
                        },
                        JsonRpcMethod::GetParameters => {
                            if let Some(animation) = animation.as_ref() {
                                respond(message.id, serde_json::json!(animation.get_parameters()));
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
                        JsonRpcMethod::OnEvent { event } => {
                            if let Some(mut animation) = animation.as_mut() {
                                animation.on_event(event);
                            }
                            respond(message.id, ());
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
            Ok(())
        }
    }
}

#[derive(Debug, Clone, FromMeta)]
struct Number {
    min: f64,
    max: f64,
    step: f64,
}

#[derive(FromField, Default, Debug)]
#[darling(default, attributes(schema_field))]
struct FieldAttributes {
    name: String,
    description: Option<String>,
    number: Option<Number>,
    color: bool,
    speed: bool,
    percentage: bool,
    enum_options: bool,
}

#[proc_macro_derive(Schema, attributes(schema_field, number))]
pub fn derive_schema(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);

    match derive_schema_inner(input) {
        Ok(output) => output,
        Err(e) => e.write_errors(),
    }
    .into()
}

fn quote_potentially_negative_number(value: f64) -> proc_macro2::TokenStream {
    if value < 0.0 {
        let value = -value;
        quote! { -(#value) }
    } else {
        quote! { #value }
    }
}

fn derive_schema_inner(
    input: proc_macro2::TokenStream,
) -> Result<proc_macro2::TokenStream, darling::Error> {
    let mut errors = darling::Error::accumulator();
    let Some(ast): Option<syn::ItemStruct> = errors.handle(
        syn::parse2(input)
            .map_err(|e| darling::Error::custom(format!("Failed to parse macro input: {e}"))),
    ) else {
        return errors.finish_with(quote! {});
    };
    let fields: proc_macro2::TokenStream = ast
        .fields
        .into_iter()
        .filter_map(|field| {
            let attrs = errors.handle(FieldAttributes::from_field(&field))?;
            let value = if let Some(number) = attrs.number {
                let min = quote_potentially_negative_number(number.min);
                let max = quote_potentially_negative_number(number.max);
                let step = number.step;
                if step.is_infinite() || step.is_nan() || !step.is_sign_positive() {
                    errors.push(darling::Error::custom("Step has to a finite positive number"));
                }
                quote! {
                    animation_api::schema::ValueSchema::Number {
                        min: (#min),
                        max: (#max),
                        step: (#step),
                    }
                }
            } else if attrs.color {
                quote! {
                    animation_api::schema::ValueSchema::Color
                }
            } else if attrs.percentage {
                quote! {
                    animation_api::schema::ValueSchema::Percentage
                }
            } else if attrs.speed {
                quote! {
                    animation_api::schema::ValueSchema::Speed
                }
            } else if attrs.enum_options {
                let ty = field.ty;
                quote! {
                    animation_api::schema::ValueSchema::Enum {
                        values: <#ty as animation_api::schema::GetEnumOptions>::enum_options(),
                    }
                }
            } else {
                errors.push(darling::Error::custom("One of 'number', 'color', 'percentage', 'speed' or 'enum' required in 'schema'"));
                return None;
            };

            let ident = field.ident;
            let name = attrs.name;
            let description = if let Some(description) = &attrs.description {
                quote! { Some(#description.to_owned()) }
            } else {
                quote! { None }
            };
            Some(quote! {
                animation_api::schema::ParameterSchema {
                    id: stringify!(#ident).to_owned(),
                    name: #name.to_owned(),
                    description: #description,
                    value: #value,
                },
            })
        })
        .collect();

    let ident = ast.ident;
    errors.finish_with(quote! {
        impl animation_api::schema::GetSchema for #ident {
            fn schema() -> animation_api::schema::ConfigurationSchema {
                animation_api::schema::ConfigurationSchema {
                    parameters: vec![
                        #fields
                    ],
                }
            }
        }
    })
}

#[derive(FromVariant, Debug)]
#[darling(attributes(schema_variant))]
struct EnumAttributes {
    ident: syn::Ident,
    name: String,
}

#[proc_macro_derive(EnumSchema, attributes(schema_variant))]
pub fn derive_enum_schema(input: TokenStream) -> TokenStream {
    let ast: syn::ItemEnum = syn::parse2(input.into()).unwrap();
    let variants: proc_macro2::TokenStream = ast
        .variants
        .into_iter()
        .map(|variant| {
            let variant = EnumAttributes::from_variant(&variant).unwrap();
            let name = variant.name;
            let ident = variant.ident;
            quote! {
                animation_api::schema::EnumOption {
                    name: #name.into(),
                    description: None,
                    value: stringify!(#ident).into(),
                },
            }
        })
        .collect();
    let ident = ast.ident;
    quote! {
        impl animation_api::schema::GetEnumOptions for #ident {
            fn enum_options() -> Vec<animation_api::schema::EnumOption> {
                vec![#variants]
            }
        }
    }
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_schema_negative_numbers() {
        let input = quote! {
            pub struct Parameters {
                #[schema_field(name = "Param", number(min = "-1.0", max = 1.0, step = 0.1))]
                param: f64,
            }
        };

        let output = derive_schema_inner(input).unwrap();

        assert_eq!(
            output.to_string(),
            quote! {
                impl animation_api::schema::GetSchema for Parameters {
                    fn schema() -> animation_api::schema::ConfigurationSchema {
                        animation_api::schema::ConfigurationSchema {
                            parameters: vec![
                                animation_api::schema::ParameterSchema {
                                    id: stringify!(param).to_owned(),
                                    name: "Param".to_owned(),
                                    description: None,
                                    value: animation_api::schema::ValueSchema::Number {
                                        min: (-(1f64)),
                                        max: (1f64),
                                        step: (0.1f64),
                                    },
                                },
                            ],
                        }
                    }
                }
            }
            .to_string()
        )
    }
}
