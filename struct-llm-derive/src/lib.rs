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
                serde_json::json!(#schema)
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

fn generate_struct_schema(fields: &Fields) -> proc_macro2::TokenStream {
    let mut properties = Vec::new();
    let mut required = Vec::new();

    match fields {
        Fields::Named(fields_named) => {
            for field in &fields_named.named {
                let field_name = field.ident.as_ref().unwrap().to_string();
                let field_schema = generate_field_schema(&field.ty);

                properties.push(quote! {
                    #field_name: #field_schema
                });

                required.push(field_name);
            }
        }
        Fields::Unnamed(_) => {
            panic!("StructuredOutput does not support tuple structs");
        }
        Fields::Unit => {
            panic!("StructuredOutput does not support unit structs");
        }
    }

    let required_fields = required.iter().map(|s| quote! { #s });

    quote! {
        {
            "type": "object",
            "properties": {
                #(#properties),*
            },
            "required": [#(#required_fields),*]
        }
    }
}

fn generate_field_schema(ty: &syn::Type) -> proc_macro2::TokenStream {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            let type_name = segment.ident.to_string();

            // Handle Vec with generic argument
            if type_name == "Vec" {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                        let item_type = infer_json_type(inner_ty);
                        return quote! {
                            {
                                "type": "array",
                                "items": {
                                    "type": #item_type
                                }
                            }
                        };
                    }
                }
                // Fallback for Vec without type info
                return quote! {
                    {
                        "type": "array",
                        "items": {}
                    }
                };
            }

            // Handle Option
            if type_name == "Option" {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                        // Return the inner type schema (Option makes field non-required)
                        return generate_field_schema(inner_ty);
                    }
                }
            }
        }
    }

    // Default case: simple type
    let type_str = infer_json_type(ty);
    quote! {
        {
            "type": #type_str
        }
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
