use convert_case::ccase;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::parse::{ParseStream, Parser};
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Fields, Meta, parenthesized};

pub fn form_data(input: DeriveInput) -> syn::Result<TokenStream> {
    let Data::Struct(data) = input.data else {
        return Err(syn::Error::new(input.span(), "Only structs are supported."));
    };

    let Fields::Named(fields) = data.fields else {
        return Err(syn::Error::new(
            data.fields.span(),
            "Only named structs are supported",
        ));
    };
    let mut fields: Vec<_> = fields
        .named
        .into_iter()
        .map(|field| -> syn::Result<_> {
            let mut immutable = false;
            for attr in field.attrs {
                if let Meta::List(meta) = attr.meta {
                    if !meta.path.is_ident("form_data") {
                        continue;
                    }
                    let parser = |input: ParseStream| {
                        let content;
                        parenthesized!(content in input);
                        let ident: syn::Ident = content.parse()?;
                        if ident != "immutable" {
                            immutable = true;
                        }
                        Ok(())
                    };
                    parser.parse2(meta.tokens)?;
                }
            }
            Ok(Field {
                name: field.ident.unwrap(),
                ty: field.ty,
                immutable: false,
            })
        })
        .collect::<Result<_, _>>()?;
    fields.retain(|field| !field.immutable);
    
    let data = input.ident;
    let update = format_ident!("{data}Update");
    let callbacks = format_ident!("{data}Callbacks");
    
    let mut variants = Vec::new();
    let mut update_arms = Vec::new();
    let mut callback_fields = Vec::new();
    let mut callback_values = Vec::new();
    let vis = input.vis;
    
    for field in fields {
        let variant = Ident::new(&ccase!(pascal, field.name.to_string()), field.name.span());
        let ty = &field.ty;
        let field = &field.name;
        variants.push(quote! {
            #variant(#ty)
        });
        update_arms.push(quote! {
            #update::#variant(value) => { self.#field = value; }
        });
        callback_fields.push(quote! {
            #field: ::yew::callback::Callback<#ty>
        });
        callback_values.push(quote! {
            #field: update.reform(#update::#variant)
        });
    }
    


    Ok(quote! {
        #vis enum #update {
            #(#variants),*
        }

        #vis struct #callbacks {
            #(#callback_fields),*
        }

        impl ::yew_forms::FormData for #data {
            type Update = #update;
            type Callbacks = #callbacks;

            fn callbacks(update: ::yew::Callback<Self::Update>) -> Self::Callbacks {
                #callbacks {
                    #(#callback_values),*
                }
            }
            fn update(&mut self, update: Self::Update) {
                match update {
                    #(#update_arms)*
                }
            }
        }
    })
}

struct Field {
    name: syn::Ident,
    ty: syn::Type,
    immutable: bool,
}
