# (DEPRECATED) picross_solver
A Rust re-implementation of the solver from Nonogrammatica, but one that uses a prior, flawed version of the solving algorithm. I probably will not come back to this.

This takes the .csv files created by saving your work in Nonogrammatica

```
cargo build --release
/target/release/picross_solver < /path/to/file.csv
```
