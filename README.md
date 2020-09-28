# Bril Toolkit? - Lesson 4: Dataflow
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
brench on the `lvn_bench` folder, grepping the output for missing or incorrect

### CLI Changes

The tool now has proper sub-commands. For running transformations:
```
cargo run -- transform -o OPTIMIZATIONS
```

For running dataflow analyses:
```
cargo run -- analyze ANALYSIS_NAME
```

### Dataflow implementation
I implemented a general dataflow "platform" by making my implementation of the
worklist algorithm generic over the transfer function, merge function, and
output datatype, with an enum for direction.

This makes it relatively easy to make new analyses since you only need to write
the merge and transfer functions for the appropriate datatype and plug them in
to the worklist algorithm. That said, both the transfer functions I've completed
so far had bugs, so who knows how easy it actually is.

I implemented a generic merge function over sets which also reduces code
duplication in a nice way.

---

I implemented a reaching definitions analysis

```
cargo run -- analyze reaching_defns
```

and a live variable analysis

```
cargo run -- analyze live
```

I also started a copy propagation analysis, but didn't finish it for reasons of
scheduling pragmatism.

I've set up a small turnt test suite for both in the `tests/df` folder.

### Notes:

The reaching definitions analysis inserts a phantom root node which declares
dummy constants to propagate the fn input definitions. Not entirely thrilled with
this solution, but it seemed to work well enough.
