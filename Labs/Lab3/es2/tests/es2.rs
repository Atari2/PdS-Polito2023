use es2::MyCycle;

//  fornendo in ingresso un iteratore con zero elementi si deve ottenere un iteratore che restituisce  zero elementi
#[test]
fn empty_iterator() {
    let v: Vec<i32> = vec![];
    let mut cycle = MyCycle::new(v.iter(), 0);
    assert_eq!(cycle.next(), None);
    assert_eq!(cycle.next(), None);
}

#[test]
fn basic_iterator() {
    let v = vec![1, 2, 3, 4];
    let cycle = MyCycle::new(v.iter(), 2);
    assert_eq!(cycle.clone().count(), 8);
    let expected = vec![1, 2, 3, 4, 1, 2, 3, 4];
    for (a, b) in cycle.zip(expected.iter()) {
        assert_eq!(*a, *b);
    }
}

/*
costruendo un MyCycle (con numero di ripetizioni finito pari a n1) a partire da un altro MyCycle
(con numero di ripetizioni finito pari a n2), si deve ottenere una sequenza che contiene n1*n2
volte la sequenza originale
 */
#[test]
fn nm_iterator() {
    let v1: Vec<i32> = vec![1, 2, 3, 4];
    let c1 = MyCycle::new(v1.iter(), 2);
    let c2 = MyCycle::new(c1, 3);
    // n*m = 2*3 = 6
    // 6*4 = 24
    assert_eq!(c2.clone().count(), 24);
    let expected: [i32; 24] = [
        1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4,
    ];
    for (a, b) in c2.zip(expected.iter()) {
        assert_eq!(*a, *b);
    }
}

/*
concatenando (tramite .chain(...) ) due MyCycle, il primo basato su una sequenza di l1
elementi con numero di ripetizioni pari a n1, il secondo basato su una sequenza di l2 elementi
con numero di ripetizioni pari a n2, si deve ottenere una sequenza di l1*n1+l2*n2 elementi
 */
#[test]
fn chain_iterator() {
    let l1 = vec![1, 2, 3, 4];
    let l2 = vec![1, 2, 3];
    let c1 = MyCycle::new(l1.iter(), 2);
    let c2 = MyCycle::new(l2.iter(), 3);
    let c3 = c1.chain(c2);
    // l1*n1+l2*n2 = 4*2+3*3 = 17
    assert_eq!(c3.clone().count(), 17);
    let expected: [i32; 17] = [1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 1, 2, 3, 1, 2, 3];
    for (a, b) in c3.zip(expected.iter()) {
        assert_eq!(*a, *b);
    }
}

/*
facendo lo zip di due MyCycle si ottiene una sequenza formata dalle coppie ordinate ottenute
dalle rispettive sequenze 
*/
#[test]
fn zip_iterator() {
    let l1 = vec![1, 2, 3, 4, 5];
    let c1 = MyCycle::new(l1.iter(), 2);
    let c2 = MyCycle::new(l1.iter(), 2);
    let c3 = c1.zip(c2);
    // l1*n1 = 5*2 = 10
    let expected: [i32; 10] = [1, 2, 3, 4, 5, 1, 2, 3, 4, 5];
    assert_eq!(c3.clone().count(), 10);
    for (idx, (a, b)) in c3.enumerate() {
        assert_eq!(*a, *b);
        assert_eq!(*a, expected[idx]);
    }
}
