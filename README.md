# ed25519-dalek ![](https://img.shields.io/crates/v/ed25519-dalek.svg) ![](https://docs.rs/ed25519-dalek/badge.svg)

Fast and efficient Rust implementation of ed25519 key generation, signing, and
verification in Rust.

# Benchmarks

On an Intel i5 Sandy Bridge running at 2.6 GHz, with TurboBoost enabled (and
also running in QubesOS with *lots* of other VMs executing), this code
achieves the following performance benchmarks:

    ∃!isisⒶwintermute:(release/0.1.0 *$)~/code/rust/ed25519 ∴ cargo bench
    Finished release [optimized] target(s) in 0.0 secs
    Running target/release/deps/ed25519-0135748522c518d8

    running 5 tests
    test ed25519::test::test_sign_verify ... ignored
    test ed25519::test::test_unmarshal_marshal ... ignored
    test ed25519::test::bench_key_generation ... bench:    54,837   ns/iter (+/- 11,613)
    test ed25519::test::bench_sign           ... bench:    69,735   ns/iter (+/- 21,902)
    test ed25519::test::bench_verify         ... bench:   183,891   ns/iter (+/- 75,304)

    test result: ok. 0 passed; 0 failed; 2 ignored; 3 measured

In comparision, the equivalent package in Golang performs as follows:

    ∃!isisⒶwintermute:(master *=)~/code/go/src/github.com/agl/ed25519 ∴ go test -bench .
    PASS
    BenchmarkKeyGeneration     20000             85880 ns/op
    BenchmarkSigning           20000             89115 ns/op
    BenchmarkVerification      10000            212585 ns/op
    ok      github.com/agl/ed25519  7.500s

Making key generation, signing, and verification a rough average of one third
faster, one fifth faster, and one eighth faster respectively.  Of course, this
is just my machine, and these results—nowhere near rigorous—should be taken
with a fistful of salt.

## Warning

[Our elliptic curve library](https://github.com/isislovecruft/curve25519-dalek)
(which this code uses) has **not** yet received sufficient peer review by
other qualified cryptographers to be considered in any way, shape, or form,
safe.

**USE AT YOUR OWN RISK**

# Documentation

Documentation is available [here](https://docs.rs/ed25519-dalek).

# Installation

To install, add the following to the dependencies section of your project's
`Cargo.toml`:

    ed25519-dalek = "^0.2"

Then, in your library or executable source, add:

    extern crate ed25519_dalek

# TODO

 * Maybe add methods to make exporting keys for backup easier.
 * Benchmark in comparison to the ed25519_ref10 code.
 * We can probably make this go even faster if we implement SHA512,
   rather than using the rust-crypto implementation whose API requires
   that we allocate memory and memzero it before mutating to store the
   digest.
