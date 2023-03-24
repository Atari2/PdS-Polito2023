use std::string::FromUtf8Error;

fn count_adjacent(i: usize, j: usize, matrix: &[&[u8]]) -> char {
    let row_count = matrix.len() - 1;
    let col_count = matrix[i].len() - 1;
    let indexes = [
        (1, 1),
        (1, -1),
        (-1, 1),
        (-1, -1),
        (0, 1),
        (0, -1),
        (1, 0),
        (-1, 0),
    ];
    let mut count = 0;
    for (x_off, y_off) in indexes {
        let x = match i.checked_add_signed(x_off) {
            Some(x) => x,
            None => continue,
        };
        let y = match j.checked_add_signed(y_off) {
            Some(y) => y,
            None => continue,
        };
        if x <= row_count && y <= col_count && matrix[x][y] == b'*' {
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
    let indexes = [
        (1, 1),
        (1, -1),
        (-1, 1),
        (-1, -1),
        (0, 1),
        (0, -1),
        (1, 0),
        (-1, 0),
    ];
    let mut bytes = minefield.into_bytes();
    for i in 0..rows {
        for j in 0..cols {
            let cell = bytes[i * cols + j];
            if cell == b'*' {
                continue;
            }
            let mut count = 0;
            for (x_off, y_off) in indexes.iter() {
                let x = match i.checked_add_signed(*x_off) {
                    Some(x) => x,
                    None => continue,
                };
                let y = match j.checked_add_signed(*y_off) {
                    Some(y) => y,
                    None => continue,
                };
                if x < rows && y < cols && bytes[x * cols + y] == b'*' {
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
