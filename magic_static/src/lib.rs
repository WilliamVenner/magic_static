#![doc = include_str!("../README.md")]
#![no_std]

pub use magic_static_macro::main;

#[doc(hidden)]
pub mod private;

#[doc(hidden)]
pub use private::*;

#[macro_export]
/// Defines new magic statics.
///
/// Magic statics are initialized manually using the `magic_static::init!` macro or `magic_static::main` attribute macro.
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
/// # #[macro_use] extern crate r#magic_static;
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
macro_rules! magic_static {
	{ $($vis:vis static $ident:ident: $ty:ty = $expr:expr;)* } => {
		compile_error!("Expected `static ref`, got `static`")
	};

	{ $($vis:vis static mut $ident:ident: $ty:ty = $expr:expr;)* } => {
		compile_error!("Expected `static ref`, got `static mut`")
	};

	{ $($vis:vis static ref $ident:ident: $ty:ty = $expr:expr;)* } => {
		$(
			$vis static $ident: $crate::MagicStatic<$ty> = $crate::MagicStatic {
				initialized: $crate::__magic_static_initialized!(),
				value: ::core::cell::UnsafeCell::new(::core::mem::MaybeUninit::uninit()),
				init: || $expr
			};
		)*
	};
}

#[macro_export]
/// Manually initializes the provided magic statics **in the specified order**.
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
/// # #[macro_use] extern crate r#magic_static;
///
/// mod foo {
/// 	magic_static! {
/// 		pub(super) static ref BAR: std::sync::Mutex<()> = std::sync::Mutex::new(());
/// 		pub(super) static ref MAGIC: usize = {
/// 			println!("Magic!");
/// 			42
/// 		};
/// 	}
/// }
///
/// fn main() {
/// 	magic_static::init! {
/// 		foo::BAR,
/// 		foo::MAGIC
/// 	}
/// }
/// ```
macro_rules! init {
	{ $($path:path),* } => {
		$($path.__init();)*
	};
}
