use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Item, ItemMod};

/// Attribute macro for declaring a test suite.
///
/// Applied to a module, it collects all `#[test]` functions and generates
/// a `suite()` function returning `(&'static str, &'static [crate::harness::TestCase])`.
///
/// The suite name defaults to the module name, or can be overridden:
/// `#[test_suite(name = "my_suite")]`
///
/// The `#[test]` attributes are stripped so they don't conflict with
/// Rust's built-in test harness (which isn't available in `no_std`).
///
/// # Example
///
/// ```ignore
/// #[embclox_test_macros::test_suite]
/// mod my_tests {
///     #[test]
///     fn it_works() {
///         assert_eq!(1 + 1, 2);
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn test_suite(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut module = parse_macro_input!(item as ItemMod);

    let mod_name = &module.ident;

    // Parse optional name = "..." from attribute
    let suite_name = if attr.is_empty() {
        mod_name.to_string()
    } else {
        let name_value: syn::MetaNameValue =
            syn::parse(attr).expect("expected: #[test_suite(name = \"...\")]");
        assert!(
            name_value.path.is_ident("name"),
            "expected: #[test_suite(name = \"...\")]"
        );
        if let syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(s),
            ..
        }) = &name_value.value
        {
            s.value()
        } else {
            panic!("expected string literal for name");
        }
    };

    let (_, items) = module
        .content
        .as_mut()
        .expect("test_suite must be applied to a module with a body (not `mod foo;`)");

    // Collect #[test] functions and strip the attribute
    let mut test_names = Vec::new();
    for item in items.iter_mut() {
        if let Item::Fn(func) = item {
            let has_test = func.attrs.iter().any(|attr| attr.path().is_ident("test"));
            if has_test {
                func.attrs.retain(|attr| !attr.path().is_ident("test"));
                test_names.push(func.sig.ident.clone());
            }
        }
    }

    // Generate suite() function
    let test_entries = test_names.iter().map(|name| {
        let name_str = name.to_string();
        quote! {
            crate::harness::TestCase { name: #name_str, func: #name }
        }
    });

    let suite_fn: Item = syn::parse2(quote! {
        pub fn suite() -> (&'static str, &'static [crate::harness::TestCase]) {
            (#suite_name, &[ #(#test_entries),* ])
        }
    })
    .expect("failed to parse generated suite function");

    items.push(suite_fn);

    quote! { #module }.into()
}
