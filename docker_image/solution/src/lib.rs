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
        .trim_end_matches(':')
        .split_whitespace();
    let width = parts.next()?.parse().ok()?;
    let height = parts.next()?.parse().ok()?;
    Some((width, height))
}

fn row_content(line: &str) -> &str {
    line.split_whitespace().last().unwrap_or("")
}

pub fn parse_board<R: BufRead>(lines: &mut R, header: &str) -> Result<Board, String> {
    let (width, height) = dimensions(header, "Anfield ").ok_or("invalid Anfield header")?;
    if width == 0 || height == 0 {
        return Err("empty Anfield".into());
    }
    let mut cells = Vec::with_capacity(height);
    for _ in 0..height {
        let mut line = String::new();
        if lines
            .read_line(&mut line)
            .map_err(|error| error.to_string())?
            == 0
        {
            return Err("incomplete Anfield".into());
        }
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
    if player != 1 && player != 2
        || x + piece.width > board.width
        || y + piece.height > board.height
    {
        return false;
    }
    let (own_a, own_b, opponent_a, opponent_b) = player_markers(player);
    let mut overlap = 0;
    for py in 0..piece.height {
        for px in 0..piece.width {
            if !piece.cells[py][px] {
                continue;
            }
            let cell = board.cells[y + py][x + px];
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

pub fn choose_placement(board: &Board, piece: &Piece, player: u8) -> Option<Placement> {
    let (_, _, opponent_a, opponent_b) = player_markers(player);
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
    let marker = line.find("exec p")? + "exec p".len();
    line[marker..]
        .chars()
        .next()?
        .to_digit(10)
        .map(|number| number as u8)
}

pub fn run<R: BufRead, W: std::io::Write>(input: R, mut output: W) -> Result<(), String> {
    let mut lines = input.lines();
    let mut player = None;
    while let Some(line) = lines.next() {
        let line = line.map_err(|error| error.to_string())?;
        if let Some(number) = player_number(&line) {
            player = Some(number);
        }
        if !line.starts_with("Anfield ") {
            continue;
        }
        let (width, height) = dimensions(&line, "Anfield ").ok_or("invalid Anfield header")?;
        let mut rows = Vec::with_capacity(height);
        for _ in 0..height {
            let row = lines
                .next()
                .ok_or("incomplete Anfield")?
                .map_err(|error| error.to_string())?;
            rows.push(row);
        }
        let board_data = rows.join("\n");
        let mut board_cursor = std::io::Cursor::new(board_data);
        let board = parse_board(&mut board_cursor, &format!("Anfield {width} {height}:"))?;
        let mut piece_header = None;
        for next in lines.by_ref() {
            let next = next.map_err(|error| error.to_string())?;
            if next.starts_with("Piece ") {
                piece_header = Some(next);
                break;
            }
            if next.starts_with("Anfield ") {
                return Err("missing Piece header".into());
            }
        }
        let piece_header = match piece_header {
            Some(header) => header,
            None => return Err("missing Piece header".into()),
        };
        let (piece_width, piece_height) =
            dimensions(&piece_header, "Piece ").ok_or("invalid Piece header")?;
        let mut piece_rows = Vec::with_capacity(piece_height);
        for _ in 0..piece_height {
            let row = lines
                .next()
                .ok_or("incomplete piece")?
                .map_err(|error| error.to_string())?;
            piece_rows.push(row);
        }
        let piece_data = piece_rows.join("\n");
        let mut piece_cursor = std::io::Cursor::new(piece_data);
        let piece = parse_piece(
            &mut piece_cursor,
            &format!("Piece {piece_width} {piece_height}:"),
        )?;
        let placement = player.and_then(|number| choose_placement(&board, &piece, number));
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

    #[test]
    fn parses_player_board_and_piece() {
        assert_eq!(player_number("$$$ exec p2 : [robot]"), Some(2));
        let mut board_input = Cursor::new("000 ....\n001 ..@.\n002 ....\n");
        let board = parse_board(&mut board_input, "Anfield 4 3:").unwrap();
        assert_eq!(board.cells[1][2], '@');
        let mut piece_input = Cursor::new(".O\nOO\n");
        let piece = parse_piece(&mut piece_input, "Piece 2 2:").unwrap();
        assert_eq!(piece.cells, vec![vec![false, true], vec![true, true]]);
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
        assert!(is_legal(&board, &one_cell, 1, 1, 1));
        assert!(!is_legal(&board, &two_cells, 1, 1, 1));
        assert!(!is_legal(&board, &one_cell, 1, 3, 1));
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
    }

    #[test]
    fn writes_exact_coordinate_format_each_turn() {
        let input = "$$$ exec p1 : [robot]\nAnfield 4 2:\n000 ...@\n001 ....\nPiece 1 1:\nO\n";
        let mut output = Vec::new();
        run(Cursor::new(input), &mut output).unwrap();
        let response = String::from_utf8(output).unwrap();
        assert_eq!(response, "3 0\n");
    }

    #[test]
    fn reads_player_two_markers_and_places_exactly_once() {
        let input = "$$$ exec p2 : [robot]\nAnfield 5 2:\n000 ..$..\n001 .....\nPiece 1 1:\nO\n";
        let mut output = Vec::new();
        run(Cursor::new(input), &mut output).unwrap();
        assert_eq!(String::from_utf8(output).unwrap(), "2 0\n");
    }

    #[test]
    fn reads_and_answers_each_turn_in_one_process() {
        let turn = "Anfield 3 2:\n000 @..\n001 ...\nPiece 1 1:\nO\n";
        let input = format!("$$$ exec p1 : [robot]\n{turn}{turn}");
        let mut output = Vec::new();
        run(Cursor::new(input), &mut output).unwrap();
        assert_eq!(String::from_utf8(output).unwrap(), "0 0\n0 0\n");
    }
}
