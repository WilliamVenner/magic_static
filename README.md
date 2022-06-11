[![crates.io](https://img.shields.io/crates/v/magic_static.svg)](https://crates.io/crates/magic_static)
[![docs.rs](https://docs.rs/magic_static/badge.svg)](https://docs.rs/magic_static/)
[![license](https://img.shields.io/crates/l/magic_static)](https://github.com/WilliamVenner/magic_static/blob/master/LICENSE)

# ✨ `magic_static`

Global singletons initialized at program start, an alternative to lazy initialization.

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
    magic_statics! {
        pub(super) static ref MAGIC: usize = {
            println!("Magic!");
            42
        };

        pub(super) static ref BAR: std::sync::Mutex<()> = std::sync::Mutex::new(());
    }
}

// You can also modularize your magic statics in a group at the module level like so:
// See `main()` for how to initialize these magic statics.
mod baz {
    magic_statics_mod! {
        pub(super) static ref MAGIC: usize = {
            println!("Magic!");
            42
        };

        pub(super) static ref BAR: std::sync::Mutex<()> = std::sync::Mutex::new(());
    }
}

// You can also decorate statics to make them magic statics
#[magic_static]
static FOO_BAR: std::thread::JoinHandle<()> = {
    std::thread::spawn(move || {
        loop { println!("HELP I CANT STOP SPINNING"); }
    })
};

#[magic_static::main(
    FOO_BAR,

    foo::MAGIC,
    foo::BAR,

    mod baz // This will initialize all magic statics in the `baz` module
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