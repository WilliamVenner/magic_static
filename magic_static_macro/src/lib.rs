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
/// // You can also modularize your magic statics like so:
/// mod baz {
/// 	magic_static! {
/// 		pub(super) static ref MAGIC: usize = {
/// 			println!("Magic!");
/// 			42
/// 		};
///
/// 		pub(super) static ref BAR: std::sync::Mutex<()> = std::sync::Mutex::new(());
/// 	}
///
/// 	#[magic_static::main(
/// 		MAGIC,
/// 		BAR
/// 	)]
/// 	// The `magic_statics!` macro (NOT `magic_static!`) can generate this function for you
/// 	pub fn magic_static() {}
/// }
///
/// #[magic_static::main(
/// 	foo::MAGIC,
/// 	foo::BAR,
/// 	mod baz // This will initialize all magic statics in `baz`
/// )]
/// fn main() {
/// 	println!("Hello, world!");
/// }
/// ```
pub fn main(attr: TokenStream, item: TokenStream) -> TokenStream {
	let mut func = syn::parse_macro_input!(item as syn::ItemFn);
	let attr = attr.to_string();

	enum MagicStatic {
		Module(syn::Path),
		Item(syn::Path),
	}
	impl quote::ToTokens for MagicStatic {
		fn to_tokens(&self, tokens: &mut quote::__private::TokenStream) {
			match self {
				MagicStatic::Module(path) => tokens.extend(quote::quote! { #path::magic_static() }),
				MagicStatic::Item(path) => tokens.extend(quote::quote! { #path.__init() }),
			}
		}
	}

	let mut magic_statics = vec![];
	for item in attr.split(",").map(|path| path.trim()) {
		if let Some(item) = item.strip_prefix("mod ").map(str::trim) {
			if item.contains("::") {
				magic_statics.push(MagicStatic::Module(syn::parse_str(item).expect("Expected path to a module containing an accessible `magic_static` function")));
			} else {
				magic_statics.push(MagicStatic::Module(syn::parse_str(&format!("self::{}", item)).expect("Expected path to a module containing an accessible `magic_static` function")));
			}
		} else {
			magic_statics.push(MagicStatic::Item(syn::parse_str(item).expect("Expected path to magic static")));
		}
	}

	func.block.stmts.insert(
		0,
		syn::parse(quote::quote! {
			{
				#(#magic_statics;)*
			}
		}.into()).expect("Internal error"),
	);

	func.into_token_stream().into()
}

#[proc_macro_attribute]
pub fn magic_static(_attr: TokenStream, item: TokenStream) -> TokenStream {
	let mut func = syn::parse_macro_input!(item as syn::ItemStatic);

	let ty = func.ty;
	let expr = func.expr;

	func.ty = Box::new(syn::parse_quote! { ::magic_static::MagicStatic<#ty> });
	func.expr = Box::new(syn::parse_quote! {
		::magic_static::MagicStatic {
			initialized: ::magic_static::__magic_static_initialized!(),
			value: ::core::cell::UnsafeCell::new(::core::mem::MaybeUninit::uninit()),
			init: || #expr
		}
	});

	func.into_token_stream().into()
}
