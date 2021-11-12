[![crates.io](https://img.shields.io/crates/v/magic_static.svg)](https://crates.io/crates/magic_static)
[![docs.rs](https://docs.rs/magic_static/badge.svg)](https://docs.rs/magic_static/)
[![license](https://img.shields.io/crates/l/magic_static)](https://github.com/WilliamVenner/magic_static/blob/master/LICENSE)

# ✨ `magic_static`

Safe, global singletons initialized at program start.

## Usage

Simply add `magic_static` as a dependency in your `Cargo.toml` to get started:

```toml
[dependencies]
magic_static = "*"
```

### `bare-metal`

If your target doesn't support atomics or threads, enable the `bare-metal` feature flag in your `Cargo.toml`:

```toml
[dependencies]
magic_static = { version = "*", features = ["bare-metal"] }
```

## Example

```rust
#[macro_use]
extern crate magic_static;

mod foo {
    magic_static! {
        pub(super) static ref MAGIC: usize = {
            println!("Magic!");
            42
        };

        pub(super) static ref BAR: std::sync::Mutex<()> = std::sync::Mutex::new(());
    }
}

// You can also modularize your magic statics like so:
mod baz {
    magic_static! {
        pub(super) static ref MAGIC: usize = {
            println!("Magic!");
            42
        };

        pub(super) static ref BAR: std::sync::Mutex<()> = std::sync::Mutex::new(());
    }

    #[magic_static::main(
        MAGIC,
        BAR
    )]
    // Must be called `magic_static`
    pub fn magic_static() {}
}

#[magic_static::main(
    foo::MAGIC,
    foo::BAR,
    mod baz // This will initialize all magic statics in `baz`
)]
fn main() {
    println!("Hello, world!");
}
```

## Comparison to [`lazy_static`](https://crates.io/crates/lazy_static)

`lazy_static`s are initialized on first-use and are targetted towards multithreaded applications.

Every time a `lazy_static` is dereferenced, it must check whether it has been initialized yet. This is usually extremely cheap, and the resulting reference can be stored for use in hot loops (for example), but in some cases you may prefer no checks at all, i.e. a more lightweight solution.

`magic_static` only performs these checks in debug builds, making it a more ergonomic choice for single-threaded and performance-critical applications.

The downside of using `magic_static` is that you must manually initialize each `magic_static` in your `main` function or somewhere appropriate. See above for an example.