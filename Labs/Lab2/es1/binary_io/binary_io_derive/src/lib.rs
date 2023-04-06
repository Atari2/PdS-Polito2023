#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

extern crate proc_macro;

use proc_macro::TokenStream;
use syn::DeriveInput;

fn check_if_attr_is_repr_c(attr: &syn::Attribute) -> bool {
    if attr.path().is_ident("repr") {
        let nested = attr.parse_args_with(syn::punctuated::Punctuated::<syn::Meta, Token![,]>::parse_terminated);
        if let Ok(nested) = nested {
            for meta in nested.iter() {
                if let syn::Meta::Path(path) = meta {
                    if path.is_ident("C") {
                        return true;
                    }
                }
            }
        }
    }
    return false;
}

#[proc_macro_derive(BinaryIO)]
pub fn binary_io_derive(input: TokenStream) -> TokenStream { 
    let input = parse_macro_input!(input as DeriveInput);
     // Build the output, possibly using quasi-quotation
    let name = &input.ident;
    let is_repr_c = input.attrs.iter().find(|a| check_if_attr_is_repr_c(a)).is_some();
    if !is_repr_c {
        let error = quote_spanned!{
            name.span() => compile_error!("Struct must be repr(C)");
        };
        return TokenStream::from(error);
    }
    let expanded = quote! {
        impl BinPack for #name {
            fn from_bytes<T: std::io::BufRead + std::io::Seek>(reader: &mut T) -> Option<Self> {
                let mut buf = [0u8; std::mem::size_of::<Self>()];
                match reader.read_exact(&mut buf) {
                    Ok(_) => unsafe { Some(std::mem::transmute(buf)) },
                    Err(_) => None,
                }
            }
            fn to_bytes<T: std::io::Write>(
                &self,
                writer: &mut std::io::BufWriter<T>,
            ) -> Result<(), std::io::Error> {
                let buf: [u8; std::mem::size_of::<Self>()] = unsafe { std::mem::transmute(*self) };
                writer.write_all(&buf)?;
                Ok(())
            }
        }
    };

    // Hand the output tokens back to the compiler
    TokenStream::from(expanded)
}