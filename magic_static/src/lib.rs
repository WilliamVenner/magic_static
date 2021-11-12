#![doc = include_str!("../README.md")]
#![no_std]

pub use magic_static_macro::main;

use core::{cell::UnsafeCell, mem::MaybeUninit};

#[doc(hidden)]
pub struct MagicStatic<T> {
	#[doc(hidden)]
	pub initialized: core::sync::atomic::AtomicBool,

	#[doc(hidden)]
	pub value: UnsafeCell<MaybeUninit<T>>,

	#[doc(hidden)]
	pub init: fn() -> T,
}
impl<T> core::ops::Deref for MagicStatic<T> {
	type Target = T;

	#[cfg_attr(debug_assertions, inline)]
	#[cfg_attr(not(debug_assertions), inline(always))]
	fn deref(&self) -> &Self::Target {
		debug_assert!(
			self.initialized.load(core::sync::atomic::Ordering::Acquire),
			"This magic static has not been initialized yet! You need to add `#[magic_static::main]` to your main function, or call `magic_static::init()` at an appropriate time."
		);
		unsafe { &*(&*self.value.get()).as_ptr() }
	}
}

unsafe impl<T> Sync for MagicStatic<T> {}

impl<T: core::fmt::Debug> core::fmt::Debug for MagicStatic<T> {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		#[cfg(debug_assertions)]
		if self.initialized.load(core::sync::atomic::Ordering::Acquire) {
			return write!(f, "uninitialized {}", core::any::type_name::<T>());
		}
		(*self).fmt(f)
	}
}

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
/// magic_static! {
/// 	pub static ref MAGIC: usize = {
/// 		println!("Magic!");
/// 		42
/// 	};
/// }
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
				initialized: ::core::sync::atomic::AtomicBool::new(false),
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
/// # #[macro_use]
/// # extern crate magic_static;
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

impl<T> MagicStatic<T> {
	#[doc(hidden)]
	#[inline]
	pub fn __init(&'static self) {
		assert!(
			!self.initialized.swap(true, core::sync::atomic::Ordering::SeqCst),
			"This magic static has already been initialized! It looks like you have multiple calls to `magic_static::init()`"
		);
		unsafe { (&mut *self.value.get()).as_mut_ptr().write((self.init)()) };
	}
}
