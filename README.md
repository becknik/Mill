# Mill in Rust

This is a [mill](https://en.wikipedia.org/wiki/Nine_men%27s_morris) implementation in Rust for the "Programmierprojekt: Mühlespiel in Rust" course in the University of Stuttgart in the summer semester of 2023.

The course is held by the FMI [FMI](https://fmi.uni-stuttgart.de/ti/teaching/s23/progproj/) and is initially taking place this semester.

## Open TODOs

Besides the TODOs in the programs text, the following parts/ rules of the game are atm not fully implemented yet:

Rules:

- If a player can't move stones any more, he has lost the party
- If a player only has stones in a closed mill, a stone can be beaten out of one closed mill

## Assignments

### Assignment 3

Just `cargo run` it :^)

### Assignment 4

Execution:

```bash
cd Mill
cargo test -- assignment
diff output.txt ../blatt_4_test_data_large/output.txt
```

Example for `input_felder.txt` © FMI Uni Stuttgart:
```
BBEEEEEBEEEEWEWWBWWEEEBE
BBEEEWEBBEWEBEEEEEEEEEEE
BEEEWWBEWEWEEEEWEEEEEBBB
BWEWEEWEBEBBEBWEWEEBEWWB
EBBBEEEWEEBEWEBEEEEEEEEE
EBEEWBWWEBBEBEWBEWEWBEWE
EEBEBWWEWEWWEEEEEEEBBBEE

```
