#![forbid(unsafe_code)]

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    parse_macro_input,
    token::{Comma, FatArrow},
    Arm, Data, DataStruct, DeriveInput, Expr, Field, Fields, Pat, Path, Type, TypePath,
};

fn get_match_arm(field: &Field) -> Arm {
    let field_ident = field.ident.as_ref().unwrap();
    let expr = match &field.ty {
        Type::Reference(ref_type) => {
            panic!("not yet implemented")
        }
        Type::Path(TypePath {
            path: Path { segments, .. },
            ..
        }) => match segments.first() {
            Some(segment) if segment.ident == "isize" => {
                quote!(Ok(sformat_dynamic::TypedValue::Int(self.#field_ident)))
            }
            Some(segment) if segment.ident == "i64" => {
                quote!(Ok(sformat_dynamic::TypedValue::Int64(self.#field_ident)))
            }
            Some(segment) if segment.ident == "i32" => {
                quote!(Ok(sformat_dynamic::TypedValue::Int32(self.#field_ident)))
            }
            Some(segment) if segment.ident == "i16" => {
                quote!(Ok(sformat_dynamic::TypedValue::Int16(self.#field_ident)))
            }
            Some(segment) if segment.ident == "i8" => {
                quote!(Ok(sformat_dynamic::TypedValue::Int8(self.#field_ident)))
            }
            Some(segment) if segment.ident == "usize" => {
                quote!(Ok(sformat_dynamic::TypedValue::Uint(self.#field_ident)))
            }
            Some(segment) if segment.ident == "u64" => {
                quote!(Ok(sformat_dynamic::TypedValue::Uint64(self.#field_ident)))
            }
            Some(segment) if segment.ident == "u32" => {
                quote!(Ok(sformat_dynamic::TypedValue::Uint32(self.#field_ident)))
            }
            Some(segment) if segment.ident == "u16" => {
                quote!(Ok(sformat_dynamic::TypedValue::Uint16(self.#field_ident)))
            }
            Some(segment) if segment.ident == "u8" => {
                quote!(Ok(sformat_dynamic::TypedValue::Uint8(self.#field_ident)))
            }
            Some(segment) if segment.ident == "f64" => {
                quote!(Ok(sformat_dynamic::TypedValue::Float64(self.#field_ident)))
            }
            Some(segment) if segment.ident == "f32" => {
                quote!(Ok(sformat_dynamic::TypedValue::Float32(self.#field_ident)))
            }
            Some(segment) if segment.ident == "bool" => {
                quote!(Ok(sformat_dynamic::TypedValue::Bool(self.#field_ident)))
            }
            _ => panic!("unhandled segment type"),
        },
        _ => panic!("unhandled field type"),
    };

    Arm {
        attrs: vec![],
        pat: Pat::Verbatim(quote!(stringify!(#field_ident))),
        guard: None,
        fat_arrow_token: FatArrow::default(),
        body: Box::new(Expr::Verbatim(expr)),
        comma: Some(Comma::default()),
    }
}

fn expand_derive_context(input: DeriveInput) -> TokenStream2 {
    if let Data::Struct(DataStruct {
        fields: Fields::Named(fields),
        ..
    }) = &input.data
    {
        let struct_name = input.ident;
        let match_arms = fields.named.iter().map(get_match_arm);

        let impl_context = quote! {
            impl<'ctxt> sformat_dynamic::Context<'ctxt> for #struct_name {
                fn get_variable<'b>(
                    &self,
                    name: sformat_dynamic::Name<'b>
                ) -> Result<
                        sformat_dynamic::TypedValue<'ctxt>,
                        sformat_dynamic::FormatError<'b>
                    >
                {
                    match name {
                        #( #match_arms )*
                        _ => Err(sformat_dynamic::FormatError::VariableNameError(name)),
                    }
                }
            }
        };

        impl_context
    } else {
        panic!("expected struct with named fields")
    }
}

#[proc_macro_derive(Context)]
pub fn derive_context(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    expand_derive_context(input).into()
}
