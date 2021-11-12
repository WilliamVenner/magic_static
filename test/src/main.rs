#[macro_use]
extern crate magic_static;

mod foo {
	magic_static! {
		pub static ref BAR: usize = {
			println!("Hello, world!");
			42
		};

		pub static ref MAGIC: std::sync::Mutex::<()> = std::sync::Mutex::new(());
	}
}

#[magic_static::main(
	foo::BAR
)]
fn main() {
	assert_eq!(*foo::BAR, 42);
	assert!(std::panic::catch_unwind(|| magic_static::init! { foo::BAR }).is_err());
	println!("Test Success");
}