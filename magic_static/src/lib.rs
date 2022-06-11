//! Global singletons initialized at program start, an alternative to lazy initialization.
//!
//! ## Usage
//!
//! Simply add `magic_static` as a dependency in your `Cargo.toml` to get started:
//!
//! ```toml
//! [dependencies]
//! magic_static = "*"
//! ```
//!
//! ### `bare-metal`
//!
//! If your target doesn't support atomics or threads, enable the `bare-metal` feature flag in your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! magic_static = { version = "*", features = ["bare-metal"] }
//! ```
//!
//! ## Example
//!
//! ```rust
//! #[macro_use]
//! extern crate magic_static;
//!
//! mod foo {
//!     magic_statics! {
//!         pub(super) static ref MAGIC: usize = {
//!             println!("Magic!");
//!             42
//!         };
//!
//!         pub(super) static ref BAR: std::sync::Mutex<()> = std::sync::Mutex::new(());
//!     }
//! }
//!
//! // You can also modularize your magic statics in a group at the module level like so:
//! // See `main()` for how to initialize these magic statics.
//! mod baz {
//!     magic_statics_mod! {
//!         pub(super) static ref MAGIC: usize = {
//!             println!("Magic!");
//!             42
//!         };
//!
//!         pub(super) static ref BAR: std::sync::Mutex<()> = std::sync::Mutex::new(());
//!     }
//! }
//!
//! // You can also decorate statics to make them magic statics
//! #[magic_static]
//! static FOO_BAR: std::thread::JoinHandle<()> = {
//!     std::thread::spawn(move || {
//!         loop { println!("HELP I CANT STOP SPINNING"); }
//!     })
//! };
//!
//! #[magic_static::main(
//!     FOO_BAR,
//!
//!     foo::MAGIC,
//!     foo::BAR,
//!
//!     mod baz // This will initialize all magic statics in the `baz` module
//! )]
//! fn main() {
//!     println!("Hello, world!");
//! }
//! ```
//!
//! ## Comparison to [`lazy_static`](https://crates.io/crates/lazy_static)
//!
//! `lazy_static`s are initialized on first-use and are targetted towards multithreaded applications.
//!
//! Every time a `lazy_static` is dereferenced, it must check whether it has been initialized yet. This is usually extremely cheap, and the resulting reference can be stored for use in hot loops (for example), but in some cases you may prefer no checks at all, i.e. a more lightweight solution.
//!
//! `magic_static` only performs these checks in debug builds, making it a more ergonomic choice for single-threaded and performance-critical applications.
//!
//! The downside of using `magic_static` is that you must manually initialize each `magic_static` in your `main` function or somewhere appropriate. See above for an example.

#![allow(clippy::needless_doctest_main)]
#![no_std]

pub use magic_static_macro::{main, magic_static};

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
/// mod foo {
///     magic_statics! {
///         pub(super) static ref MAGIC: usize = {
///             println!("Magic!");
///             42
///         };
///
///         pub(super) static ref BAR: std::sync::Mutex<()> = std::sync::Mutex::new(());
///     }
/// }
///
/// // You can also modularize your magic statics in a group at the module level like so:
/// // See `main()` for how to initialize these magic statics.
/// mod baz {
///     magic_statics_mod! {
///         pub(super) static ref MAGIC: usize = {
///             println!("Magic!");
///             42
///         };
///
///         pub(super) static ref BAR: std::sync::Mutex<()> = std::sync::Mutex::new(());
///     }
/// }
///
/// // You can also decorate statics to make them magic statics
/// #[magic_static]
/// static FOO_BAR: std::thread::JoinHandle<()> = {
///     std::thread::spawn(move || {
///         loop { println!("HELP I CANT STOP SPINNING"); }
///     })
/// };
///
/// #[magic_static::main(
///     FOO_BAR,
///
///     foo::MAGIC,
///     foo::BAR,
///
///     mod baz // This will initialize all magic statics in the `baz` module
/// )]
/// fn main() {
///     println!("Hello, world!");
/// }
/// ```
macro_rules! magic_statics {
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
/// **You can only have one of these per module (scope)** - if you want to initialize magic statics in a group, define a `magic_static` function in your module yourself! (See the example)
///
/// # Example
///
/// ```rust
/// mod foo {
///     // Note the use of `magic_statics_mod!` rather than `magic_static!` here
///     magic_statics_mod! {
///         pub(super) static ref MAGIC: usize = {
///             println!("Magic!");
///             42
///         };
///
///         pub(super) static ref BAR: std::sync::Mutex<()> = std::sync::Mutex::new(());
///     }
///
///     // Will generate the following:
///     /*
///     #[magic_static::main(
///         MAGIC,
///         BAR
///     )]
///     pub fn magic_static() {}
///     */
/// }
///
/// #[magic_static::main(
///     mod foo // This will initialize all magic statics in `foo`
/// )]
/// fn main() {
///     println!("Hello, world!");
/// }
/// ```
macro_rules! magic_statics_mod {
	{ $($vis:vis static ref $ident:ident: $ty:ty = $expr:expr;)* } => {
		$crate::magic_statics!($($vis static ref $ident: $ty = $expr;)*);

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
/// Does nothing to a magic static if it has already been initialized.
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
///     magic_static! {
///         pub(super) static ref BAR: std::sync::Mutex<()> = std::sync::Mutex::new(());
///         pub(super) static ref MAGIC: usize = {
///             println!("Magic!");
///             42
///         };
///     }
/// }
///
/// // You can also modularize your magic statics like so:
/// mod baz {
///     magic_static! {
///         pub(super) static ref MAGIC: usize = {
///             println!("Magic!");
///             42
///         };
///
///         pub(super) static ref BAR: std::sync::Mutex<()> = std::sync::Mutex::new(());
///     }
///
///     #[magic_static::main(
///         MAGIC,
///         BAR
///     )]
///     // Must be called `magic_static`
///     pub fn magic_static() {}
/// }
///
/// fn main() {
///     magic_static::init! {
///         foo::BAR,
///         foo::MAGIC,
///         mod baz // This will initialize all magic statics in `baz`
///     }
/// }
/// ```
macro_rules! init {
	() => {};

	(mod $($path:ident)::+) => {
		$($path)::+::magic_static()
	};

	(mod $($path:ident)::+, $($tail:tt)*) => {{
		$($path)::+::magic_static();
		$crate::init!($($tail)*);
	}};

	($path:path) => {
		$path.__init()
	};

	($path:path, $($tail:tt)*) => {{
		$path.__init();
		$crate::init!($($tail)*);
	}};
}
