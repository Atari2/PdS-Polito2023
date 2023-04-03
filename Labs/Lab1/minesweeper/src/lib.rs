use std::string::FromUtf8Error;

fn calculate_offsets(i: usize, j: usize, rows: usize, cols: usize) -> impl Iterator<Item = (usize, usize)> {

    let offsets: &[(isize, isize)] = &[
        (-1, -1),
        (-1, 1),
        (-1, 0),
        (0, -1),
        (1, 1),
        (1, -1),
        (1, 0),
        (0, 1)
    ];

    offsets.iter().filter_map(move |(x_off, y_off)| {
        let (x, y) = (i.checked_add_signed(*x_off), j.checked_add_signed(*y_off));
        match (x, y) {
            (Some(x), Some(y)) if x < rows && y < cols => Some((x, y)),
            _ => None,
        }
    })
}

fn count_adjacent(i: usize, j: usize, matrix: &[&[u8]]) -> char {
    let offsets = calculate_offsets(i, j, matrix.len(), matrix[i].len());
    let mut count = 0;
    for (x, y) in offsets {
        if matrix[x][y] == b'*' {
            count += 1;
        }
    }
    if count == 0 {
        ' '
    } else {
        (b'0' + count) as char
    }
}

pub fn annotate2(minefield: String, rows: usize, cols: usize) -> Result<String, FromUtf8Error> {
    let mut bytes = minefield.into_bytes();
    for i in 0..rows {
        for j in 0..cols {
            let cell = bytes[i * cols + j];
            if cell == b'*' {
                continue;
            }
            let mut count = 0;
            let offsets = calculate_offsets(i, j, rows, cols);
            for (x, y) in offsets {
                if bytes[x * cols + y] == b'*' {
                    count += 1;
                }
            }
            if count > 0 {
                bytes[i * cols + j] = b'0' + count;
            }
        }
    }
    String::from_utf8(bytes)
}

pub fn annotate(minefield: &[&str]) -> Vec<String> {
    let mut result = vec![];
    let bytes_minefield = minefield.iter().map(|s| s.as_bytes()).collect::<Vec<_>>();
    for (i, row) in bytes_minefield.iter().enumerate() {
        let mut current_row = String::new();
        for (j, cell) in row.iter().enumerate() {
            if *cell == b'*' {
                current_row.push('*');
            } else {
                current_row.push(count_adjacent(i, j, &bytes_minefield));
            }
        }
        result.push(current_row);
    }
    result
}
