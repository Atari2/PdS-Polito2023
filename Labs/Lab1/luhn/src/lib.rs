/// Check a Luhn checksum.
pub fn is_valid(code: &str) -> bool {
    let mut sum = 0;
    let code_str = code.chars().filter(|c| !c.is_whitespace()).collect::<String>();
    if code_str.len() <= 1 {
        return false;
    }
    for (i, c) in code_str.chars().rev().enumerate() {
        if !c.is_ascii_digit() {
            return false;
        }
        let digit = c.to_digit(10);
        if let Some(mut digit) = digit {
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
    sum % 10 == 0
}
