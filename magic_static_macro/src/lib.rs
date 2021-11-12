use proc_macro::TokenStream;
use quote::ToTokens;

#[proc_macro_attribute]
/// An attribute that can be attached to your main function which initializes magic statics **in the specified order**.
///
/// # Panics
///
/// This will panic if any of the magic statics have already been initialized.
///
/// # Safety
///
/// The following behaviour is considered undefined:
///
/// * Initializing magic statics from multiple threads concurrently.
/// * Spawning new threads and accessing magic statics during initialization from them.
/// * Interior mutability of magic statics where the mutability is not synchronized across multiple threads (e.g. with a Mutex or RwLock.) This is not a problem for single-threaded applications.
///
/// # Example
///
/// ```rust
/// # #[macro_use]
/// # extern crate magic_static;
///
/// mod foo {
/// 	magic_static! {
/// 		pub(super) static ref MAGIC: usize = {
/// 			println!("Magic!");
/// 			42
/// 		};
///
/// 		pub(super) static ref BAR: std::sync::Mutex::<()> = std::sync::Mutex::new(());
/// 	}
/// }
///
/// #[magic_static::main(
/// 	foo::MAGIC,
///     foo::BAR
/// )]
/// fn main() {
/// 	println!("Hello, world!");
/// }
/// ```
pub fn main(attr: TokenStream, item: TokenStream) -> TokenStream {
	let mut func = syn::parse_macro_input!(item as syn::ItemFn);

	let paths: Vec<syn::Path> = syn::parse_macro_input!(attr as syn::AttributeArgs)
		.into_iter()
		.map(|meta| match meta {
			syn::NestedMeta::Meta(syn::Meta::Path(path)) => path,
			_ => panic!("Expected path"),
		})
		.collect();

	func.block.stmts.insert(
		0,
		syn::parse(quote::quote! { { #(#paths.__init();)* } }.into()).expect("Internal error"),
	);

	func.into_token_stream().into()
}
