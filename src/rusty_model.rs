use convert_case::{Case, Casing};
use darling::{ast::Data, util::Ignored, FromDeriveInput, FromField};
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{ext::IdentExt, parse_macro_input, Ident, LitStr, Path, Type};

#[derive(FromField)]
#[darling(attributes(rusty_model))]
struct Field {
    ident: Option<Ident>,
    ty: Type,

    #[darling(default)]
    findable: bool,
}

#[derive(FromDeriveInput)]
#[darling(attributes(rusty_model))]
struct Options {
    ident: Ident,
    data: Data<Ignored, Field>,

    service: Path,
    #[darling(default)]
    belongs_to: Vec<LitStr>,
    #[darling(default)]
    has_many: Vec<LitStr>,
}

pub fn handle(token: TokenStream) -> TokenStream {
    let input = parse_macro_input!(token);
    let options = Options::from_derive_input(&input).expect("Wrong options");

    let struct_name = options.ident;

    let fields = options
        .data
        .take_struct()
        .expect("Only struct is supported");

    let id_type = fields
        .iter()
        .find_map(|field| {
            if let Some(ident) = field.ident.as_ref() {
                if ident.unraw() == "id" {
                    return Some(field.ty.clone());
                }
            }
            None
        })
        .expect("Id not found");

    let find_by = fields.iter().filter_map(|field| {
        let field_name = field.ident.clone().expect("No field name found");
        let find_by_field_name = Ident::new(
            &format!("find_by_{}", field_name.unraw()),
            Span::call_site(),
        );
        let field_type = field.ty.clone();

        if field.findable {
            Some(quote! {
                pub fn #find_by_field_name(value: &#field_type) -> Vec<Self> {
                    Self::all()
                        .into_iter()
                        .filter(|a| &a.#field_name == value)
                        .collect()
                }
            })
        } else {
            None
        }
    });

    let belongs_to = options.belongs_to.iter().map(|a| {
        let value = a.value();
        let function_name = Ident::new(value.to_case(Case::Snake).as_str(), Span::call_site());
        let type_name = Ident::new(value.to_case(Case::Pascal).as_str(), Span::call_site());
        let id_name = Ident::new(
            format!("{}_id", value.to_case(Case::Snake)).as_str(),
            Span::call_site(),
        );
        quote! {
            pub fn #function_name(&self) -> Option<#type_name> {
                #type_name::find(&self.#id_name)
            }
        }
    });

    let has_many = options.has_many.iter().map(|a| {
        let value = a.value();
        let function_name = Ident::new(
            format!("{}_list", value.to_case(Case::Snake)).as_str(),
            Span::call_site(),
        );
        let type_name = Ident::new(value.to_case(Case::Pascal).as_str(), Span::call_site());
        let find_by_id_name = Ident::new(
            format!(
                "find_by_{}_id",
                struct_name.to_string().to_case(Case::Snake)
            )
            .as_str(),
            Span::call_site(),
        );
        quote! {
            pub fn #function_name(&self) -> Vec<#type_name> {
                #type_name::#find_by_id_name(&self.id)
            }
        }
    });

    let destroy_children = options.has_many.iter().map(|a| {
        let value = a.value();
        let list_function_name = Ident::new(
            format!("{}_list", value.to_case(Case::Snake)).as_str(),
            Span::call_site(),
        );
        quote! {
            for child in self.#list_function_name().into_iter() {
                child.destroy()?;
            }
        }
    });

    let service = options.service;

    let output = quote! {
        impl #struct_name {
            #(#find_by)*
            #(#belongs_to)*
            #(#has_many)*

            pub fn all() -> Vec<Self> {
                #service::all()
            }

            #[allow(clippy::ptr_arg)]
            pub fn find(id: &#id_type) -> Option<Self> {
                #service::find(id)
            }

            pub fn save(self) -> std::result::Result<Self, #service::SaveError> {
                #service::save(self)
            }

            pub fn destroy(self) -> std::result::Result<(), #service::DestroyError> {
                #(#destroy_children)*
                #service::destroy(&self.id)
            }
        }
    };
    output.into()
}
