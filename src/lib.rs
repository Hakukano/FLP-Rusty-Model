mod rusty_model;

use proc_macro::TokenStream;

#[proc_macro_derive(RustyModel, attributes(rusty_model))]
pub fn rusty_model(tokens: TokenStream) -> TokenStream {
    rusty_model::handle(tokens)
}
