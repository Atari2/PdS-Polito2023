use std::string::FromUtf8Error;

fn count_adjacent(i: i64, j: i64, matrix: &[&[u8]]) -> u8 {
    let row_count = (matrix.len() - 1) as i64;
    let col_count = (matrix[i as usize].len() - 1) as i64;
    let indexes = vec![
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
        let x = i + x_off;
        let y = j + y_off;
        if x >= 0
            && x <= row_count
            && y >= 0
            && y <= col_count
            && matrix[x as usize][y as usize] == b'*'
        {
            count += 1;
        }
    }
    count
}

pub fn annotate2(minefield: String, rows: usize, cols: usize) -> Result<String, FromUtf8Error> {
    let indexes: Vec<(i64, i64)> = vec![
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
                let x = i as i64 + x_off;
                let y = j as i64 + y_off;
                if x >= 0
                    && x < rows as i64
                    && y >= 0
                    && y < cols as i64
                    && bytes[(x * cols as i64 + y) as usize] == b'*'
                {
                    count += 1;
                }
            }
            if count == 0 {
                continue;
            }
            bytes[i * cols + j] = b'0' + count;
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
                let count = count_adjacent(i as i64, j as i64, &bytes_minefield);
                if count == 0 {
                    current_row.push(' ');
                }
                else {
                    current_row.push((b'0' + count) as char);
                }
            }
        }
        result.push(current_row);
    }
    result
}
