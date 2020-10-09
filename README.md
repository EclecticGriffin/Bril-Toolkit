# Bril Toolkit? - Lesson 5: SSA
## Griffin Berlstein

### Build Instructions

```
cargo build
```

### Test Suite
```
cd tests
bash tests.sh
```

This will run turnt over all the folders with turnt tests defined and will run
brench on the `lvn_bench` folder, grepping the output for missing or incorrect. This has also been modified to transform a number of programs to SSA form and then check that they are indeed SSA. There are also brench pipelines to test correctness with respect to SSA and the "roundtrip" transformation.

### CLI Changes

Added two new transformations under the optimizations category, namely
`to_ssa` and `from_ssa`.

```
cargo run -- transform -o to_ssa
```

```
cargo run -- transform -o from_ssa
```

### Notes
I have a small test suite working, though I suspect there are likely bugs somewhere that I haven't fully cleaned up. A lot of this turned out to be somewhat tricky in small ways. So the code's become a real mess, which I'll work on cleaning up later.
