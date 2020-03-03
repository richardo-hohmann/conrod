use std;

use proc_macro2;
use syn;
use utils;
// The implementation for `WidgetStyle`.
//
// This generates an accessor method for every field in the struct
pub fn impl_widget_style(ast: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let crate_tokens = Some(syn::Ident::new("_conrod", proc_macro2::Span::call_site()));
    let params = params(ast).unwrap();
    let impl_tokens = impl_tokens(&params, crate_tokens);
    let dummy_const = &params.dummy_const;
    quote! {
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const #dummy_const: () = {
            extern crate conrod_core as _conrod;
            #impl_tokens
        };
    }
}

// The implementation for `WidgetStyle_`.
//
// The same as `WidgetStyle` but only for use within the conrod crate itself.
pub fn impl_widget_style_(ast: &syn::DeriveInput) -> proc_macro2::TokenStream {
   // let crate_tokens = syn::Ident::from(syn::token::CapSelf::default());
    let crate_tokens= None;
    let params = params(ast).unwrap();
    let impl_tokens = impl_tokens(&params, crate_tokens);
    let dummy_const = &params.dummy_const;
    quote! {
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const #dummy_const: () = {
            #impl_tokens
        };
    }
}

fn impl_tokens(params: &Params, crate_tokens: Option<syn::Ident>) -> proc_macro2::TokenStream {
    let Params {
        ref impl_generics,
        ref ty_generics,
        ref where_clause,
        ref ident,
        ref fields,
        ..
    } = *params;
    let getter_methods = fields
        .iter()
        .map(|&FieldParams { ref default, ref ty, ref ident }| {
            quote! {
                /// Retrieves the value, falling back to a default values in the following order:
                ///
                /// 1. If the field is `None`, falls back to the style stored within the `Theme`.
                /// 2. If there are no style defaults for the widget in the theme, or if the
                ///    default field is also `None`, falls back to the expression specified within
                ///    the field's `#[conrod(default = "expr")]` attribute.
                ///
                /// *This method was generated by the `#[conrod(default = "expr")]` attribute
                /// associated with the `#[derive(WidgetStyle)]` attribute.*
                pub fn #ident(&self, theme: &#crate_tokens::Theme) -> #ty {
                    self.#ident
                        .or_else(|| {
                            theme.widget_style::<Self>()
                                .and_then(|default| default.style.#ident)
                        })
                        .unwrap_or_else(|| #default)
                }
            }
        });

    quote! {
        impl #impl_generics #ident #ty_generics #where_clause {
            #( #getter_methods )*
        }
    }
}

#[derive(Debug)]
struct Params {
    impl_generics: proc_macro2::TokenStream,
    ty_generics: proc_macro2::TokenStream,
    where_clause: proc_macro2::TokenStream,
    ident: proc_macro2::TokenStream,
    fields: Vec<FieldParams>,
    dummy_const: proc_macro2::TokenStream,
}

#[derive(Debug)]
struct FieldParams {
    default: proc_macro2::TokenStream,
    ty: proc_macro2::TokenStream,
    ident: proc_macro2::TokenStream,
}

fn params(ast: &syn::DeriveInput) -> Result<Params, Error> {

    // Ensure we are deriving for a struct.
    let body = match ast.data {
        syn::Data::Struct(ref body) => body,
        _ => return Err(Error::NotStruct),
    };

    // We can only derive `WidgetStyle` for structs with fields.
    match body.fields {
        syn::Fields::Named(_) => {},
        syn::Fields::Unnamed(_) => return Err(Error::TupleStruct),
        syn::Fields::Unit => return Err(Error::UnitStruct),
    };

    // For each field in the struct, create a method
    //
    // Produce an iterator yielding `Tokens` for each method.
    let fields = body.fields
        .iter()
        .filter_map(|field| {

            let attr_elems = utils::conrod_attrs(&field.attrs);

            let mut item = None;
            'attrs: for nested_items in attr_elems {
                for nested_item in nested_items{
                    if let syn::NestedMeta::Meta(ref meta_item) = nested_item {
                        item = Some(meta_item.clone());
                        break 'attrs;
                    }
                }
            }

            let item = match item {
                Some(item) => item,
                None => return None,
            };

            let literal = match item {
                syn::Meta::NameValue(syn::MetaNameValue{ref path, ref lit,..}) if path.is_ident("default") => lit,
                ref item => return Some(Err(Error::UnexpectedMetaItem(item.clone()))),
            };

            let default: syn::Expr = match *literal {
                syn::Lit::Str(ref litstr) => litstr.clone().parse().unwrap(),
                ref literal => return Some(Err(Error::UnexpectedLiteral(literal.clone()))),
            };
            let ident = match field.ident {
                Some(ref ident) => ident,
                None => return Some(Err(Error::UnnamedStructField)),
            };

            let ty = {
                let path = match field.ty {
                    syn::Type::Path(syn::TypePath{ref path,..}) => path,
                    _ => return Some(Err(Error::NonOptionFieldTy)),
                };

                // TODO: Add handling for `std::option::Option` (currently only handles `Option`).
                let path_segment = match path.segments.len() {
                    1 => &path.segments[0],
                    _ => return Some(Err(Error::NonOptionFieldTy)),
                };

                if path_segment.ident != "Option" {
                    return Some(Err(Error::NonOptionFieldTy));
                }

                let angle_bracket_data = match path_segment.arguments {
                    syn::PathArguments::AngleBracketed(ref data) => data,
                    _ => return Some(Err(Error::NonOptionFieldTy)),
                };

                let ty = match angle_bracket_data.args.len() {
                    1 => angle_bracket_data.args.first().unwrap(),
                    _ => return Some(Err(Error::NonOptionFieldTy)),
                };

                ty
            };

            let params = FieldParams {
                default: quote!(#default),
                ty: quote!(#ty),
                ident: quote!(#ident),
            };

            Some(Ok(params))
        })
        .collect::<Result<_, _>>()?;

    let dummy_const = syn::Ident::new(&format!("_IMPL_WIDGET_STYLE_FOR_{}", ast.ident), proc_macro2::Span::call_site());
    let ident = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    Ok(Params {
        impl_generics: quote!(#impl_generics),
        ty_generics: quote!(#ty_generics),
        where_clause: quote!(#where_clause),
        ident: quote!(#ident),
        fields: fields,
        dummy_const: quote!(#dummy_const),
    })
}

#[derive(Debug)]
enum Error {
    NotStruct,
    TupleStruct,
    UnitStruct,
    UnexpectedLiteral(syn::Lit),
    UnexpectedMetaItem(syn::Meta),
    UnnamedStructField,
    NonOptionFieldTy,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match *self {
            Error::NotStruct =>
                "`#[derive(WidgetStyle)]` is only defined for structs",
            Error::TupleStruct =>
                "#[derive(WidgetCommon)]` is not defined for tuple structs",
            Error::UnitStruct =>
                "#[derive(WidgetCommon)]` is not defined for unit structs",
            Error::UnexpectedLiteral(ref _lit) =>
                "Found unexpected literal in `conrod` attribute",
            Error::UnexpectedMetaItem(ref _item) =>
                "Found unexpected meta item in `conrod` attribute",
            Error::UnnamedStructField =>
                "Cannot use #[conrod(default = \"foo\")] attribute on unnamed fields",
            Error::NonOptionFieldTy =>
                "Cannot use #[conrod(default = \"foo\")] attribute on non-`Option` fields"
        };
        write!(f, "{}", s)
    }
}
