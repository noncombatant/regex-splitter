# `regex-splitter`

This crate provides an `Iterator` that scans a `Read` looking for instances of a
`Regex` delimiter, and yields the bytes between instances of the delimiter.

It is similar in purpose to
[`regex-chunker`](https://github.com/d2718/regex-chunker), but differs in that
it does less (no `async`, for example), so its implementation is simpler and has
fewer dependencies.

## Building And Running

`cargo test` runs the tests, as usual. There is an example program,
src/bin/readme.rs, that is copied and adapted from the same program in the
`regex-chunker` source repository. To run it:

```sh
cargo run --bin readme < some-file.txt
```

To compare the performance of this and other crates, you’ll want to build them
in release mode:

```sh
cargo build --release --bin readme
time ./target/release/readme < some-file.txt
...
```

So far the performance of `regex-splitter` and `regex-chunker` seems comparable.