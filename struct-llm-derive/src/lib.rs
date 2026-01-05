use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields, Lit};

/// Derive macro for `StructuredOutput` trait.
///
/// # Attributes
///
/// - `name`: Tool name (required) - e.g., `name = "final_answer"`
/// - `description`: Tool description (required) - e.g., `description = "Returns the final answer"`
///
/// # Example
///
/// ```ignore
/// #[derive(Serialize, Deserialize, StructuredOutput)]
/// #[structured_output(
///     name = "create_npc",
///     description = "Creates an NPC character"
/// )]
/// struct NPCData {
///     name: String,
///     age: u32,
///     backstory: String,
/// }
/// ```
#[proc_macro_derive(StructuredOutput, attributes(structured_output))]
pub fn derive_structured_output(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Extract attributes
    let (tool_name, tool_description) = parse_attributes(&input);

    // Generate JSON schema from struct fields
    let schema = generate_schema(&input.data);

    let expanded = quote! {
        impl #impl_generics struct_llm::StructuredOutput for #name #ty_generics #where_clause {
            fn tool_name() -> &'static str {
                #tool_name
            }

            fn tool_description() -> &'static str {
                #tool_description
            }

            fn json_schema() -> serde_json::Value {
                #schema
            }
        }
    };

    TokenStream::from(expanded)
}

fn parse_attributes(input: &DeriveInput) -> (String, String) {
    let mut tool_name = None;
    let mut tool_description = None;

    for attr in &input.attrs {
        if !attr.path().is_ident("structured_output") {
            continue;
        }

        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("name") {
                let value = meta.value()?;
                let s: Lit = value.parse()?;
                if let Lit::Str(lit_str) = s {
                    tool_name = Some(lit_str.value());
                }
            } else if meta.path.is_ident("description") {
                let value = meta.value()?;
                let s: Lit = value.parse()?;
                if let Lit::Str(lit_str) = s {
                    tool_description = Some(lit_str.value());
                }
            }
            Ok(())
        });
    }

    let tool_name = tool_name.expect("missing #[structured_output(name = \"...\")] attribute");
    let tool_description = tool_description.expect("missing #[structured_output(description = \"...\")] attribute");

    (tool_name, tool_description)
}

fn generate_schema(data: &Data) -> proc_macro2::TokenStream {
    match data {
        Data::Struct(data_struct) => generate_struct_schema(&data_struct.fields),
        Data::Enum(_) => {
            panic!("StructuredOutput can only be derived for structs, not enums");
        }
        Data::Union(_) => {
            panic!("StructuredOutput can only be derived for unions");
        }
    }
}

/// Generates a TokenStream that produces a serde_json::Value at runtime
fn generate_field_schema_tokens(ty: &syn::Type) -> proc_macro2::TokenStream {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            let type_name = segment.ident.to_string();

            // Handle Vec with generic argument
            if type_name == "Vec" {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                        // Check if inner type is a primitive
                        if is_primitive_type(inner_ty) {
                            let item_type = infer_json_type(inner_ty);
                            return quote! {
                                {
                                    let mut items_schema = serde_json::Map::new();
                                    items_schema.insert("type".to_string(), serde_json::Value::String(#item_type.to_string()));

                                    let mut schema = serde_json::Map::new();
                                    schema.insert("type".to_string(), serde_json::Value::String("array".to_string()));
                                    schema.insert("items".to_string(), serde_json::Value::Object(items_schema));
                                    serde_json::Value::Object(schema)
                                }
                            };
                        } else {
                            // For custom types (structs), call their json_schema() at runtime
                            // This requires the inner type to implement StructuredOutput
                            return quote! {
                                {
                                    let inner_schema = <#inner_ty as struct_llm::StructuredOutput>::json_schema();
                                    let mut schema = serde_json::Map::new();
                                    schema.insert("type".to_string(), serde_json::Value::String("array".to_string()));
                                    schema.insert("items".to_string(), inner_schema);
                                    serde_json::Value::Object(schema)
                                }
                            };
                        }
                    }
                }
                // Fallback for Vec without type info
                return quote! {
                    {
                        let mut schema = serde_json::Map::new();
                        schema.insert("type".to_string(), serde_json::Value::String("array".to_string()));
                        schema.insert("items".to_string(), serde_json::Value::Object(serde_json::Map::new()));
                        serde_json::Value::Object(schema)
                    }
                };
            }

            // Handle Option
            if type_name == "Option" {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                        // Return the inner type schema (Option makes field non-required)
                        return generate_field_schema_tokens(inner_ty);
                    }
                }
            }
        }
    }

    // Default case: simple type
    let type_str = infer_json_type(ty);
    quote! {
        {
            let mut schema = serde_json::Map::new();
            schema.insert("type".to_string(), serde_json::Value::String(#type_str.to_string()));
            serde_json::Value::Object(schema)
        }
    }
}

fn generate_struct_schema(fields: &Fields) -> proc_macro2::TokenStream {
    let mut field_insertions = Vec::new();
    let mut required = Vec::new();

    match fields {
        Fields::Named(fields_named) => {
            for field in &fields_named.named {
                let field_name = field.ident.as_ref().unwrap().to_string();
                let field_schema = generate_field_schema_tokens(&field.ty);

                field_insertions.push(quote! {
                    properties.insert(#field_name.to_string(), #field_schema);
                });

                // Only add to required if NOT an Option type
                if !is_option_type(&field.ty) {
                    required.push(field_name);
                }
            }
        }
        Fields::Unnamed(_) => {
            panic!("StructuredOutput does not support tuple structs");
        }
        Fields::Unit => {
            panic!("StructuredOutput does not support unit structs");
        }
    }

    quote! {
        {
            let mut properties = serde_json::Map::new();
            #(#field_insertions)*

            let required_fields: Vec<serde_json::Value> = vec![
                #(serde_json::Value::String(#required.to_string())),*
            ];

            let mut schema = serde_json::Map::new();
            schema.insert("type".to_string(), serde_json::Value::String("object".to_string()));
            schema.insert("properties".to_string(), serde_json::Value::Object(properties));
            schema.insert("required".to_string(), serde_json::Value::Array(required_fields));
            serde_json::Value::Object(schema)
        }
    }
}

/// Check if a type is a known primitive that maps directly to a JSON type
fn is_primitive_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            let type_name = segment.ident.to_string();
            matches!(
                type_name.as_str(),
                "String" | "str" |
                "i8" | "i16" | "i32" | "i64" | "i128" |
                "u8" | "u16" | "u32" | "u64" | "u128" |
                "isize" | "usize" |
                "f32" | "f64" |
                "bool"
            )
        } else {
            false
        }
    } else {
        false
    }
}

/// Check if a type is Option<T>
fn is_option_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            segment.ident == "Option"
        } else {
            false
        }
    } else {
        false
    }
}

fn infer_json_type(ty: &syn::Type) -> &'static str {
    // Simple type inference - extract the last segment of the path
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            let type_name = segment.ident.to_string();

            return match type_name.as_str() {
                "String" | "str" => "string",
                "i8" | "i16" | "i32" | "i64" | "i128" |
                "u8" | "u16" | "u32" | "u64" | "u128" |
                "isize" | "usize" => "integer",
                "f32" | "f64" => "number",
                "bool" => "boolean",
                "Vec" => "array",
                "HashMap" | "BTreeMap" => "object",
                _ => {
                    // Check if it's an Option
                    if type_name == "Option" {
                        // For Option types, we need to look at the inner type
                        if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                            if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                                return infer_json_type(inner_ty);
                            }
                        }
                    }
                    // Default to string for custom types
                    "string"
                }
            };
        }
    }

    "string" // Default fallback
}
