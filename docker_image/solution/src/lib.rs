use std::io::BufRead;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Vec<char>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Piece {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Vec<bool>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Placement {
    pub x: usize,
    pub y: usize,
}

fn dimensions(line: &str, prefix: &str) -> Option<(usize, usize)> {
    let mut parts = line
        .strip_prefix(prefix)?
        .trim()
        .trim_end_matches(':')
        .split_whitespace();
    let width = parts.next()?.parse().ok()?;
    let height = parts.next()?.parse().ok()?;
    Some((width, height))
}

fn row_content(line: &str) -> &str {
    line.split_whitespace().last().unwrap_or("")
}

fn read_line<R: BufRead>(input: &mut R) -> Result<Option<String>, String> {
    let mut line = String::new();
    match input
        .read_line(&mut line)
        .map_err(|error| error.to_string())?
    {
        0 => Ok(None),
        _ => Ok(Some(line)),
    }
}

fn is_column_header(line: &str, width: usize) -> bool {
    let columns = row_content(line).as_bytes();
    columns.len() == width
        && columns
            .iter()
            .enumerate()
            .all(|(index, column)| *column == b'0' + (index % 10) as u8)
}

pub fn parse_board<R: BufRead>(lines: &mut R, header: &str) -> Result<Board, String> {
    let (width, height) = dimensions(header, "Anfield ").ok_or("invalid Anfield header")?;
    if width == 0 || height == 0 {
        return Err("empty Anfield".into());
    }
    let mut cells = Vec::with_capacity(height);
    let mut first_line = String::new();
    if lines
        .read_line(&mut first_line)
        .map_err(|error| error.to_string())?
        == 0
    {
        return Err("incomplete Anfield".into());
    }
    let mut pending_line = if is_column_header(&first_line, width) {
        None
    } else {
        Some(first_line)
    };
    for _ in 0..height {
        let line = match pending_line.take() {
            Some(line) => line,
            None => {
                let mut line = String::new();
                if lines
                    .read_line(&mut line)
                    .map_err(|error| error.to_string())?
                    == 0
                {
                    return Err("incomplete Anfield".into());
                }
                line
            }
        };
        let row: Vec<char> = row_content(&line).chars().collect();
        if row.len() != width {
            return Err("Anfield row has incorrect width".into());
        }
        cells.push(row);
    }
    Ok(Board {
        width,
        height,
        cells,
    })
}

pub fn parse_piece<R: BufRead>(lines: &mut R, header: &str) -> Result<Piece, String> {
    let (width, height) = dimensions(header, "Piece ").ok_or("invalid Piece header")?;
    if width == 0 || height == 0 {
        return Err("empty piece".into());
    }
    let mut cells = Vec::with_capacity(height);
    for _ in 0..height {
        let mut line = String::new();
        if lines
            .read_line(&mut line)
            .map_err(|error| error.to_string())?
            == 0
        {
            return Err("incomplete piece".into());
        }
        let row: Vec<bool> = row_content(&line).chars().map(|cell| cell != '.').collect();
        if row.len() != width {
            return Err("piece row has incorrect width".into());
        }
        cells.push(row);
    }
    Ok(Piece {
        width,
        height,
        cells,
    })
}

fn player_markers(player: u8) -> (char, char, char, char) {
    if player == 1 {
        ('@', 'a', '$', 's')
    } else {
        ('$', 's', '@', 'a')
    }
}

pub fn is_legal(board: &Board, piece: &Piece, player: u8, x: usize, y: usize) -> bool {
    if !matches!(player, 1 | 2)
        || x > board.width
        || y > board.height
        || piece.width > board.width - x
        || piece.height > board.height - y
    {
        return false;
    }
    let (own_a, own_b, opponent_a, opponent_b) = player_markers(player);
    let mut overlap = 0;
    for py in 0..piece.height {
        for px in 0..piece.width {
            let occupied = match piece.cells.get(py).and_then(|row| row.get(px)) {
                Some(&occupied) => occupied,
                None => return false,
            };
            if !occupied {
                continue;
            }
            let cell = match board.cells.get(y + py).and_then(|row| row.get(x + px)) {
                Some(&cell) => cell,
                None => return false,
            };
            if cell == opponent_a || cell == opponent_b {
                return false;
            }
            if cell == own_a || cell == own_b {
                overlap += 1;
            }
        }
    }
    overlap == 1
}

fn open_space_scores(board: &Board) -> Vec<Vec<i64>> {
    let mut scores = vec![vec![0; board.width]; board.height];
    for y in 0..board.height {
        for x in 0..board.width {
            let top = y.saturating_sub(2);
            let bottom = y.saturating_add(2).min(board.height - 1);
            let left = x.saturating_sub(2);
            let right = x.saturating_add(2).min(board.width - 1);
            let mut empty_cells = 0;
            for row in top..=bottom {
                for column in left..=right {
                    if board.cells[row][column] == '.' {
                        empty_cells += 1;
                    }
                }
            }
            scores[y][x] = empty_cells;
        }
    }
    scores
}

pub fn choose_placement(board: &Board, piece: &Piece, player: u8) -> Option<Placement> {
    if board.width == 0 || board.height == 0 || piece.width == 0 || piece.height == 0 {
        return None;
    }
    let (_, _, opponent_a, opponent_b) = player_markers(player);
    let open_space = open_space_scores(board);
    let mut best: Option<(i64, Placement)> = None;
    for y in 0..=board.height.saturating_sub(piece.height) {
        for x in 0..=board.width.saturating_sub(piece.width) {
            if !is_legal(board, piece, player, x, y) {
                continue;
            }
            let mut score = 0i64;
            for py in 0..piece.height {
                for px in 0..piece.width {
                    if !piece.cells[py][px] {
                        continue;
                    }
                    let bx = x + px;
                    let by = y + py;
                    score += open_space[by][bx] * 3;
                    for dy in -1isize..=1 {
                        for dx in -1isize..=1 {
                            if dx == 0 && dy == 0 {
                                continue;
                            }
                            let nx = bx as isize + dx;
                            let ny = by as isize + dy;
                            if nx >= 0
                                && ny >= 0
                                && (nx as usize) < board.width
                                && (ny as usize) < board.height
                            {
                                let neighbor = board.cells[ny as usize][nx as usize];
                                if neighbor == opponent_a || neighbor == opponent_b {
                                    score += 10;
                                }
                            }
                        }
                    }
                }
            }
            let center_x = (x * 2 + piece.width) as i64;
            let center_y = (y * 2 + piece.height) as i64;
            score -= (center_x - board.width as i64).abs() + (center_y - board.height as i64).abs();
            let placement = Placement { x, y };
            if best.map_or(true, |(best_score, _)| score > best_score) {
                best = Some((score, placement));
            }
        }
    }
    best.map(|(_, placement)| placement)
}

pub fn player_number(line: &str) -> Option<u8> {
    line.split_whitespace()
        .find_map(|token| token.strip_prefix('p')?.parse::<u8>().ok())
        .filter(|number| matches!(number, 1 | 2))
}

pub fn run<R: BufRead, W: std::io::Write>(mut input: R, mut output: W) -> Result<(), String> {
    let mut player = None;
    while let Some(line) = read_line(&mut input)? {
        if let Some(number) = player_number(&line) {
            player = Some(number);
        }
        if !line.starts_with("Anfield ") {
            continue;
        }
        let player = player.ok_or("missing player number")?;
        let board = parse_board(&mut input, &line)?;
        let piece_header = loop {
            let next = read_line(&mut input)?.ok_or("missing Piece header")?;
            if next.starts_with("Piece ") {
                break next;
            }
            if next.starts_with("Anfield ") {
                return Err("missing Piece header".into());
            }
        };
        let piece = parse_piece(&mut input, &piece_header)?;
        let placement = choose_placement(&board, &piece, player);
        if let Some(placement) = placement {
            writeln!(output, "{} {}", placement.x, placement.y)
                .map_err(|error| error.to_string())?;
        } else {
            writeln!(output, "0 0").map_err(|error| error.to_string())?;
        }
        output.flush().map_err(|error| error.to_string())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn response_coordinates(response: &str) -> Vec<(usize, usize)> {
        assert!(response.ends_with('\n'));
        response
            .lines()
            .map(|line| {
                let mut coordinates = line.split_whitespace();
                let x = coordinates.next().unwrap().parse().unwrap();
                let y = coordinates.next().unwrap().parse().unwrap();
                assert!(coordinates.next().is_none());
                (x, y)
            })
            .collect()
    }

    #[test]
    fn parses_captured_real_engine_turn() {
        let transcript = include_str!("../tests/fixtures/real_map00_turn.txt");
        let mut reader = Cursor::new(transcript);
        let mut exec_line = String::new();
        reader.read_line(&mut exec_line).unwrap();
        let player = player_number(&exec_line).unwrap();
        assert_eq!(player, 1);

        let mut board_header = String::new();
        reader.read_line(&mut board_header).unwrap();
        let board = parse_board(&mut reader, board_header.trim_end()).unwrap();
        assert_eq!((board.width, board.height), (20, 15));
        assert_eq!(board.cells[2][9], '@');
        assert_eq!(board.cells[12][9], '$');

        let mut piece_header = String::new();
        reader.read_line(&mut piece_header).unwrap();
        let piece = parse_piece(&mut reader, piece_header.trim_end()).unwrap();
        assert_eq!((piece.width, piece.height), (3, 3));

        let mut output = Vec::new();
        run(Cursor::new(transcript), &mut output).unwrap();
        let response = String::from_utf8(output).unwrap();
        let placements = response_coordinates(&response);
        assert_eq!(placements.len(), 1);
        assert!(is_legal(
            &board,
            &piece,
            player,
            placements[0].0,
            placements[0].1
        ));
    }

    #[test]
    fn requires_exactly_one_own_cell_and_no_opponent_cells() {
        let board = Board {
            width: 5,
            height: 3,
            cells: vec![
                vec!['.', '.', '.', '.', '.'],
                vec!['.', '@', 'a', '$', '.'],
                vec!['.', '.', '.', '.', '.'],
            ],
        };
        let one_cell = Piece {
            width: 1,
            height: 1,
            cells: vec![vec![true]],
        };
        let two_cells = Piece {
            width: 2,
            height: 1,
            cells: vec![vec![true, true]],
        };
        let three_cells = Piece {
            width: 3,
            height: 1,
            cells: vec![vec![true, true, true]],
        };
        assert!(!is_legal(&board, &one_cell, 1, 0, 1));
        assert!(is_legal(&board, &one_cell, 1, 1, 1));
        assert!(!is_legal(&board, &two_cells, 1, 1, 1));
        assert!(!is_legal(&board, &one_cell, 1, 3, 1));
        assert!(!is_legal(&board, &three_cells, 1, 1, 1));
    }

    #[test]
    fn rejects_placements_outside_board_edges() {
        let board = Board {
            width: 3,
            height: 2,
            cells: vec![vec!['@', '.', '.'], vec!['.', '.', '.']],
        };
        let piece = Piece {
            width: 2,
            height: 1,
            cells: vec![vec![true, true]],
        };
        assert!(!is_legal(&board, &piece, 1, 2, 0));
        assert!(!is_legal(&board, &piece, 1, 0, 2));
        assert!(!is_legal(&board, &piece, 1, usize::MAX, 0));
    }

    #[test]
    fn allows_placements_touching_each_board_edge() {
        let board = Board {
            width: 3,
            height: 3,
            cells: vec![
                vec!['.', '@', '.'],
                vec!['@', '.', '@'],
                vec!['.', '@', '.'],
            ],
        };
        let piece = Piece {
            width: 1,
            height: 1,
            cells: vec![vec![true]],
        };
        assert!(is_legal(&board, &piece, 1, 1, 0));
        assert!(is_legal(&board, &piece, 1, 0, 1));
        assert!(is_legal(&board, &piece, 1, 2, 1));
        assert!(is_legal(&board, &piece, 1, 1, 2));
    }

    #[test]
    fn writes_exact_coordinate_format_each_turn() {
        let input =
            "$$$ exec p1 : [robot]\nAnfield 4 2:\n    0123\n000 ....\n001 ...@\nPiece 1 1:\nO\n";
        let mut output = Vec::new();
        run(Cursor::new(input), &mut output).unwrap();
        let response = String::from_utf8(output).unwrap();
        assert_eq!(response, "3 1\n");
    }

    #[test]
    fn parses_captured_real_player_two_turn() {
        let transcript = include_str!("../tests/fixtures/real_map00_p2_turn.txt");
        let mut reader = Cursor::new(transcript);
        let mut exec_line = String::new();
        reader.read_line(&mut exec_line).unwrap();
        let player = player_number(&exec_line).unwrap();
        assert_eq!(player, 2);

        let mut board_header = String::new();
        reader.read_line(&mut board_header).unwrap();
        let board = parse_board(&mut reader, board_header.trim_end()).unwrap();
        assert_eq!(board.cells[1][10], 'a');
        assert_eq!(board.cells[12][9], '$');

        let mut piece_header = String::new();
        reader.read_line(&mut piece_header).unwrap();
        let piece = parse_piece(&mut reader, piece_header.trim_end()).unwrap();
        assert_eq!((piece.width, piece.height), (2, 1));

        let mut output = Vec::new();
        run(Cursor::new(transcript), &mut output).unwrap();
        let response = String::from_utf8(output).unwrap();
        let placements = response_coordinates(&response);
        assert_eq!(placements.len(), 1);
        assert!(is_legal(
            &board,
            &piece,
            player,
            placements[0].0,
            placements[0].1
        ));
    }

    #[test]
    fn reads_and_answers_each_turn_in_one_process() {
        let turn = include_str!("../tests/fixtures/real_map00_turn.txt");
        let input = format!("{turn}{turn}");
        let mut output = Vec::new();
        run(Cursor::new(input), &mut output).unwrap();
        let response = String::from_utf8(output).unwrap();
        let placements = response_coordinates(&response);
        assert_eq!(placements.len(), 2);
        assert_eq!(placements[0], placements[1]);
    }

    #[test]
    fn returns_zero_coordinates_when_no_legal_move_exists() {
        let input = "$$$ exec p1 : [robot]\nAnfield 3 2:\n  012\n000 ...\n001 ...\nPiece 1 1:\nO\n";
        let mut output = Vec::new();
        run(Cursor::new(input), &mut output).unwrap();
        assert_eq!(String::from_utf8(output).unwrap(), "0 0\n");
    }
}
