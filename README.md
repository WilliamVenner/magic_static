# ✨ `magic_static` (`no_std`)

Safe, global singletons initialized at program start.

# Usage

Simply add `magic_static` as a dependency to get started:

[`Cargo.toml`]()

```toml
[dependencies]
magic_static = "*"
```

# Example

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

#[magic_static::main(
    foo::MAGIC,
    foo::BAR
)]
fn main() {
    println!("Hello, world!");
}
```

### Comparison to [`lazy_static`](https://crates.io/crates/lazy_static)

`lazy_static`s are initialized on first-use and are heavily targetted towards multithreaded applications.

Every time a `lazy_static` is dereferenced, it must check whether it has been initialized yet. This is usually extremely cheap, and the resulting reference can be stored for use in hot loops (for example), but in some cases you may prefer no checks at all, i.e. a more lightweight solution.

`magic_static` only performs these checks in debug builds, making it a more ergonomic choice for single-threaded and performance-critical applications.

The downside of using `magic_static` is that you must manually initialize each `magic_static` in your `main` function or somewhere appropriate. See above for an example.