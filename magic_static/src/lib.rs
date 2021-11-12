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
/// 	// Must be called `magic_static`
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
/// The same as `magic_static!` but automatically generates the module-level `magic_static` function for you:
///
/// ```rust
/// mod foo {
/// 	// Note the use of `magic_statics!` rather than `magic_static!` here
/// 	magic_statics! {
/// 		pub(super) static ref MAGIC: usize = {
/// 			println!("Magic!");
/// 			42
/// 		};
///
/// 		pub(super) static ref BAR: std::sync::Mutex<()> = std::sync::Mutex::new(());
/// 	}
///
/// 	// If we used the `magic_static!` macro instead, we'd have to write this ourselves:
/// 	/*
/// 	#[magic_static::main(
/// 		MAGIC,
/// 		BAR
/// 	)]
/// 	pub fn magic_static() {}
/// 	*/
/// }
///
/// #[magic_static::main(
/// 	mod foo // This will initialize all magic statics in `foo`
/// )]
/// fn main() {
/// 	println!("Hello, world!");
/// }
/// ```
macro_rules! magic_statics {
	{ $($vis:vis static ref $ident:ident: $ty:ty = $expr:expr;)* } => {
		$crate::magic_static!($($vis static ref $ident: $ty = $expr;)*);

		#[doc(hidden)]
		#[inline]
		pub fn magic_static() {
			$crate::init! {
				$($ident),*
			}
		}
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
/// 	// Must be called `magic_static`
/// 	pub fn magic_static() {}
/// }
///
/// fn main() {
/// 	magic_static::init! {
/// 		foo::BAR,
/// 		foo::MAGIC,
/// 		mod baz // This will initialize all magic statics in `baz`
/// 	}
/// }
/// ```
macro_rules! init {
	() => {};

	(mod $($path:ident)::+) => {
		$($path)::+::magic_static()
	};

	(mod $($path:ident)::+, $($tail:tt)*) => {
		$($path)::+::magic_static();
		$crate::init!($($tail)*);
	};

	($path:path) => {
		$path.__init()
	};

	($path:path, $($tail:tt)*) => {
		$path.__init();
		$crate::init!($($tail)*);
	};
}
