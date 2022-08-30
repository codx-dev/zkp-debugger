# Dusk PLONK debugger

To run:

```shell
cargo run --bin pdb -- ./assets/test.cdf
```

### Example

First we build the binaries

```shell
cargo build --release
```

Then we generate a CDF file from the existing circuit

```shell
CDF_OUTPUT=target/naive.cdf cargo run --release --manifest-path example-plonk-circuit/Cargo.toml
```

And finally we run the debugger application with the generated file

```shell
cargo run --release --bin pdb -- target/naive.cdf
```
