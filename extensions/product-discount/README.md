## Note this is an extension that uses the product discount function 

This method ensures discount amounts are always in whole numbers, eliminating the possibility of decimals after applying a discount. They will need to consistently generate a discount code using this Discount Function, rather than using our native discount code creator.


# Shopify Function development with Rust

## Dependencies

- [Install Rust](https://www.rust-lang.org/tools/install)
  - On Windows, Rust requires the [Microsoft C++ Build Tools](https://docs.microsoft.com/en-us/windows/dev-environment/rust/setup). Be sure to select the _Desktop development with C++_ workload when installing them.
- Install [`cargo-wasi`](https://bytecodealliance.github.io/cargo-wasi/)
  - `cargo install cargo-wasi`

## Building the function

You can build this individual function using `cargo wasi`.

```shell
cargo wasi build --release
```

The Shopify CLI `build` command will also execute this, based on the configuration in `shopify.extension.toml`.
