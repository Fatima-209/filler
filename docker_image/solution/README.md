# Filler robot

Build from the running container, in `/filler`:

```sh
cd solution
cargo build --release
cd ..
./linux_game_engine -f maps/map01 -p1 solution/target/release/filler_robot -p2 linux_robots/bender
```

The robot reads each Anfield and piece from standard input and writes one `X Y` coordinate line per turn. If it has no legal placement, it writes `0 0`.

Run the unit tests from `/filler/solution` with `cargo test`.
