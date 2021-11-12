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

#[magic_static::main(
	foo::BAR
)]
fn main() {
	assert_eq!(*foo::BAR, 42);
	assert!(std::panic::catch_unwind(|| magic_static::init! { foo::BAR }).is_err());

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