#[macro_use]
extern crate magic_static;

mod foo {
	magic_static! {
		pub static ref BAR: usize = {
			println!("Hello, world!");
			42
		};

		pub static ref MAGIC: std::sync::Mutex::<usize> = std::sync::Mutex::new(69);
	}
}

mod some_module {
	magic_static! {
		pub static ref WOW: usize = {
			println!("Wow!");
			420
		};
	}

	#[magic_static::main(WOW)]
	pub fn magic_static() {}
}

mod other_module {
	magic_static! {
		pub static ref WOW: usize = {
			println!("Wow 2!");
			420
		};

		pub static ref OOH: usize = 0;
		pub static ref OK: usize = 1;
	}

	#[magic_static::main(WOW)]
	pub fn magic_static() {}
}

mod auto_module {
	magic_statics! {
		pub static ref WOW: usize = {
			println!("Wow 3!");
			420
		};

		pub static ref OOH: usize = 0;
		pub static ref OK: usize = 1;
	}
}

magic_static! {
	pub static ref TOP_LEVEL: usize = {
		println!("TOP_LEVEL!");
		1337
	};
}

#[magic_static::main(
	TOP_LEVEL,
	foo::BAR,
	mod some_module
)]
fn main() {
	assert_eq!(*foo::BAR, 42);
	assert!(std::panic::catch_unwind(|| magic_static::init! { foo::BAR }).is_err());

	magic_static::init! {
		mod crate::other_module,
		crate::other_module::OOH,
		self::other_module::OK,

		mod auto_module
	}

	{
		let barrier = std::sync::Arc::new(std::sync::Barrier::new(3));
		let barrier_a = barrier.clone();
		let barrier_b = barrier.clone();
		let barrier_c = barrier.clone();
		let a = std::thread::spawn(move || {
			barrier_a.wait();
			magic_static::init! { foo::MAGIC }
		});
		let b = std::thread::spawn(move || {
			barrier_b.wait();
			magic_static::init! { foo::MAGIC }
		});
		let c = std::thread::spawn(move || {
			barrier_c.wait();
			magic_static::init! { foo::MAGIC }
		});
		let n = if a.join().is_err() { 1 } else { 0 } + if b.join().is_err() { 1 } else { 0 } + if c.join().is_err() { 1 } else { 0 };
		assert_eq!(n, 2);
	}

	println!("{magic:?} {magic} {magic:x}", magic = foo::BAR);
	println!("{:?}", foo::MAGIC);

	println!("Test Success");
}