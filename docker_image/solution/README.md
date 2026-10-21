# Filler robot

This Rust program is a player for the Filler game. The game engine sends the robot its player number, current Anfield and a piece on standard input. The robot replies on standard output with the piece's top-left coordinates as `X Y\n`. A turn with no legal move is answered with `0 0\n`.

## Run in the supplied Docker container

The supplied game files are in `/filler`, and this solution is mounted at `/filler/solution`. In Docker Desktop, start the supplied container if it is stopped, then open its **Exec** terminal. If using a host terminal instead, enter the container with `docker exec -it modest_carson sh` (replace `modest_carson` with your container name).

Build and test the robot inside the container:

```sh
cd /filler/solution/docker_image/solution
/usr/local/cargo/bin/cargo test
/usr/local/cargo/bin/cargo build --release
```

The executable is created at `/filler/solution/docker_image/solution/target/release/filler_robot`.

## Play a game

Run the engine from `/filler`. The supplied Linux engine and robot paths use the `linux_` prefix:

```sh
cd /filler
./linux_game_engine -f maps/map01 -p1 solution/docker_image/solution/target/release/filler_robot -p2 linux_robots/bender
```

To test the robot as Player 2, swap the player arguments:

```sh
./linux_game_engine -f maps/map01 -p1 linux_robots/bender -p2 solution/docker_image/solution/target/release/filler_robot
```

The engine prints final scores and a winner when a game completes. A valid full-game check should finish without an illegal-move error, timeout or crash. The robot must overlap exactly one of its own cells, must not overlap the opponent, and must stay within the board.

## Audit checks

Run five games for each required matchup, alternating the robot between `-p1` and `-p2` each time. Count a win only when the winner line corresponds to the player slot containing `filler_robot`:

| Map | Opponent | Required wins |
| --- | --- | ---: |
| `maps/map00` | `linux_robots/wall_e` | 4 of 5 or more |
| `maps/map01` | `linux_robots/h2_d2` | 4 of 5 or more |
| `maps/map02` | `linux_robots/bender` | 4 of 5 or more |

For the large-map timeout check, add `-t 1` to each `map02` engine command; a game should finish without a timeout. To check the supplied engine and opponent robots independently, run:

```sh
./linux_game_engine -f maps/map01 -p1 linux_robots/bender -p2 linux_robots/terminator
```

## Implementation and tests

The robot skips the engine's column-number header when parsing the Anfield, reads the declared board and piece dimensions, and parses each once per turn. Placement validation requires exactly one overlap with the player's territory, rejects opponent overlap, and checks board boundaries. Placement selection favors open space while considering nearby opponents and board center.

Run the test suite with `/usr/local/cargo/bin/cargo test` from the solution directory. Tests in the `src/lib.rs` test module cover captured real-engine P1 and P2 input fixtures in `tests/fixtures/`, board and piece parsing, repeated turns, output formatting, 0/1/2 own-cell overlaps, opponent overlap, board edges, and the no-legal-move response.
