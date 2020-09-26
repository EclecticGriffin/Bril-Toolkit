# Bril Toolkit? - Lesson 3: DCE and LVN
## Griffin Berlstein

I've implemented LVN and Trivial DCE (local & global). I also added a number of
extensions to LVN including copy propagation, algebraic identities, and a few
other identities such as adding zero to an unknown value or multiplying an
unknown value by one.

I tested my lvn implementation on a small battery of bril program in the
`lvn_bench` folder and used brench to both evaluate speed and check correctness.
I also added a few specific tests to try to check edge cases and demonstrate
functionality, such as `boolean_fun.bril` and `fn_call.bril`.

For testing overall functionality I used `turnt` and a folder of tests called
`lvn`. For these files, I used a mix of lvn with dce and lvn without.

To make the tool usable I've added a bunch of command line flags that can be used
to enable various optimization passes.

For trivial local & global dce:
```
cargo run -- -o g_tdce l_tdce
```

For LVN:
```
cargo run -- -o lvn
```
This will also run both local & global dce after running LVN.


To only run LVN, without running dce after:
```
cargo run -- -o solo_lvn
```

To remove orphan blocks:
```
cargo run -- -o orph
```

### Build Instructions
```
cargo build
```
