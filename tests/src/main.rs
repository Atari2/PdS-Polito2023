fn main() {
    let v = vec![6, 10, 15, 12, 15];
    let names = vec!["Aria", "Prova", "Terra"];
    let vnames = v.iter().copied().zip(names.iter().copied()).collect::<Vec<_>>();
    for (x, y) in vnames {
        println!("{}: {}", x, y);
    }
    if v.iter().all(|x| *x >= 6 && *x <= 15) {
        println!("All elements are in range");
    } else {
        println!("Some elements are not in range");
    }
}
