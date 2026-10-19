# Filler robot

The supplied container mounts this repository at `/filler/solution`. Build and test from the Docker Exec terminal:

```sh
cd /filler/solution/docker_image/solution
/usr/local/cargo/bin/cargo test
/usr/local/cargo/bin/cargo build --release
cd /filler
./linux_game_engine -f maps/map01 -p1 solution/docker_image/solution/target/release/filler_robot -p2 linux_robots/bender
```

The robot reads each Anfield and piece once from standard input and writes one `X Y` coordinate line per turn. If it has no legal placement, it writes `0 0`. It rejects out-of-bounds placements, opponent overlap, and moves that do not overlap exactly one own cell.

The parser regression tests use captured P1 and P2 turns from real games in `tests/fixtures/`. Placement search favors open surrounding space while considering the opponent and board center.
