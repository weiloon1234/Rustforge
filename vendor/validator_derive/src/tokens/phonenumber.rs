use quote::quote;

use crate::types::Phonenumber;
use crate::utils::{quote_code, quote_message, CrateName};

pub fn phonenumber_tokens(
    crate_name: &CrateName,
    phonenumber: Phonenumber,
    field_name: &proc_macro2::TokenStream,
    field_name_str: &str,
) -> proc_macro2::TokenStream {
    let country_field = phonenumber.field;
    let message = quote_message(phonenumber.message);
    let code = quote_code(crate_name, phonenumber.code, "phonenumber");

    quote! {
        if !#crate_name::validate_phonenumber_by_country_iso2(&#field_name, &self.#country_field) {
            #code
            #message
            err.add_param(::std::borrow::Cow::from("field"), &stringify!(#country_field));
            err.add_param(::std::borrow::Cow::from("country"), &self.#country_field);
            err.add_param(::std::borrow::Cow::from("value"), &#field_name);
            errors.add(#field_name_str, err);
        }
    }
}
