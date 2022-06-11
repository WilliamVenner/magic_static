use core::{cell::UnsafeCell, mem::MaybeUninit};

#[macro_export]
#[doc(hidden)]
#[cfg(not(feature = "bare-metal"))]
macro_rules! __magic_static_initialized {
	() => {
		::core::sync::atomic::AtomicU8::new(0)
	};
}

#[macro_export]
#[doc(hidden)]
#[cfg(feature = "bare-metal")]
macro_rules! __magic_static_initialized {
	() => {
		::core::cell::UnsafeCell::new(false)
	};
}

#[doc(hidden)]
pub struct MagicStatic<T> {
	#[doc(hidden)]
	#[cfg(not(feature = "bare-metal"))]
	pub initialized: core::sync::atomic::AtomicU8,

	#[doc(hidden)]
	#[cfg(feature = "bare-metal")]
	pub initialized: UnsafeCell<bool>,

	#[doc(hidden)]
	pub value: UnsafeCell<MaybeUninit<T>>,

	#[doc(hidden)]
	pub init: fn() -> T,
}
impl<T> MagicStatic<T> {
	#[inline]
	#[cfg(not(feature = "bare-metal"))]
	fn initialized(&self) -> bool {
		self.initialized.load(core::sync::atomic::Ordering::Acquire) == 2
	}

	#[inline]
	#[cfg(feature = "bare-metal")]
	fn initialized(&self) -> bool {
		unsafe { *self.initialized.get() }
	}

	#[doc(hidden)]
	#[inline]
	pub fn __init(&'static self) {
		unsafe {
			#[cfg(not(feature = "bare-metal"))]
			match self.initialized.compare_exchange(0, 1, core::sync::atomic::Ordering::SeqCst, core::sync::atomic::Ordering::SeqCst) {
				Ok(0) => {
					(&mut *self.value.get()).as_mut_ptr().write((self.init)());
					self.initialized.store(2, core::sync::atomic::Ordering::SeqCst);
				},

				Err(0) | Err(1) => {
					// Spin and wait
					while self.initialized.load(core::sync::atomic::Ordering::Relaxed) != 2 {
						core::hint::spin_loop();
					}
				},

				Err(2) => {},

				code => unreachable!("{:?}", code)
			}

			#[cfg(feature = "bare-metal")]
			if !*self.initialized.get() {
				*self.initialized.get() = true;
				(&mut *self.value.get()).as_mut_ptr().write((self.init)());
			}
		}
	}
}
impl<T> core::ops::Deref for MagicStatic<T> {
	type Target = T;

	#[cfg_attr(debug_assertions, inline)]
	#[cfg_attr(not(debug_assertions), inline(always))]
	fn deref(&self) -> &Self::Target {
		debug_assert!(
			self.initialized(),
			"This magic static has not been initialized yet! You need to add `#[magic_static::main]` to your main function, or call `magic_static::init()` at an appropriate time."
		);
		unsafe { &*(&*self.value.get()).as_ptr() }
	}
}

unsafe impl<T> Sync for MagicStatic<T> {}

macro_rules! impl_fmt {
	{ $($fmt:path),+ } => {
		$(
			impl<T: $fmt> $fmt for MagicStatic<T> {
				fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
					(**self).fmt(f)
				}
			}
		)+
	};
}
impl_fmt! {
	core::fmt::Debug,
	core::fmt::Display,
	core::fmt::Binary,
	core::fmt::LowerHex,
	core::fmt::UpperHex,
	core::fmt::Octal,
	core::fmt::Pointer,
	core::fmt::LowerExp,
	core::fmt::UpperExp
}
