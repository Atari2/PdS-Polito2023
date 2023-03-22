use clap::Parser;
/*
La funzione deve convertire in maiuscolo il primo carattere di ogni parola che compone il
testo s, ignorando eventuali altri caratteri maiuscoli al suo interno.
Le parole sono separate da uno o più spazi
*/

fn capitalize(s: &str) -> String {
    let mut result = String::new();
    let mut first = true;
    for c in s.chars() {
        if first {
            result.push_str(c.to_uppercase().to_string().as_str());
            first = false;
        } else {
            first = c.is_whitespace();
            result.push(c);
        }
    }
    result
}

#[derive(Parser, Debug)]
struct Args {
    input: String,
}

/*
 stringa con più di una parola
 stringa con una sola parola senza spazi
 stringa con caratteri accentati all’inizio di parola
 stringa vuota
 stringa con più spazi
*/
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multiple_words() {
        assert_eq!(capitalize("ciao mondo"), "Ciao Mondo");
    }
    #[test]
    fn test_one_word() {
        assert_eq!(capitalize("ciao"), "Ciao");
    }
    #[test]
    fn test_with_accents() {
        assert_eq!(
            capitalize("il sole è sorto, àèìòù"),
            "Il Sole È Sorto, Àèìòù"
        );
    }
    #[test]
    fn test_empty_string() {
        assert_eq!(capitalize(""), "");
    }
    #[test]
    fn test_multiple_spaces() {
        assert_eq!(capitalize("ciao   mondo"), "Ciao   Mondo");
    }
}

fn main() {
    let args = Args::parse();
    println!("{}", capitalize(&args.input));
}
