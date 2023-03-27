/// Check a Luhn checksum.
pub fn is_valid(code: &str) -> bool {
    let mut sum = 0;
    let mut count = 0;
    for (i, c) in code
        .chars()
        .filter(|c| !c.is_whitespace())
        .rev()
        .enumerate()
    {
        count += 1;
        if !c.is_ascii_digit() {
            return false;
        }
        if let Some(mut digit) = c.to_digit(10) {
            if i % 2 == 1 {
                digit *= 2;
                sum += if digit > 9 { digit - 9 } else { digit };
            } else {
                sum += digit;
            }
        } else {
            return false;
        }
    }
    if count <= 1 {
        return false;
    }
    sum % 10 == 0
}
