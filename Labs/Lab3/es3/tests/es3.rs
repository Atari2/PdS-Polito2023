use es3::{Hour, Calendar};

fn calendar_or_panic(calstr: &str) -> Calendar {
    match Calendar::from_string(calstr) {
        Ok(cal) => cal,
        Err(e) => {
            panic!("Error: {:?}", e);
        }
    }
}

//  uno e entrambi i calendari senza appuntamenti
#[test]
fn no_appointments() {
    let c1 = calendar_or_panic("8:30\n18:30");
    let mut slots = c1.find_slots(30);
    let slot = slots.next().unwrap();
    assert_eq!(slot.0, Hour::new(8, 30));
    assert_eq!(slot.1, Hour::new(18, 30));
    assert_eq!(slots.next(), None);
}

#[test]
fn both_no_appointments() {
    let c1 = calendar_or_panic("8:30\n18:30");
    let c2 = calendar_or_panic("9:00\n20:00");
    let slots = c1.find_slots(30).chain(c2.find_slots(30));
    assert_eq!(slots.count(), 2);
}

//  un calendario pieno
#[test]
fn full_calendar() {
    let c1 = calendar_or_panic("8:30\n18:30\n8:30\n10:00\n10:00\n12:00\n12:00\n14:00\n14:00\n16:00\n16:00\n18:30");
    let slots = c1.find_slots(30);
    assert_eq!(slots.count(), 0);
}

//  un calendario con orari liberi ma di lunghezza insufficiente
#[test]
fn not_enough_time() {
    let c1 = calendar_or_panic("8:30\n18:30\n9:00\n10:00\n10:30\n12:00\n12:30\n14:00\n14:30\n16:00\n16:30\n18:00");
    let slots = c1.find_slots(60);
    assert_eq!(slots.count(), 0);
}

//  tempo disponibile a inizio e fine giornata
#[test]
fn available_at_start_and_end() {
    let c1 = calendar_or_panic("8:30\n18:30\n9:00\n18:00");
    let mut slots = c1.find_slots(30);
    let slot = slots.next().unwrap();
    assert_eq!(slot.0, Hour::new(8, 30));
    assert_eq!(slot.1, Hour::new(9, 0));
    let slot = slots.next().unwrap();
    assert_eq!(slot.0, Hour::new(18, 0));
    assert_eq!(slot.1, Hour::new(18, 30));
    assert_eq!(slots.next(), None);
}

//  un intervallo disponibile lungo esattamente quanto richiesto
#[test]
fn exact_duration() {
    let c1 = calendar_or_panic("8:30\n18:30\n8:30\n14:30\n15:30\n18:30");
    let mut slots = c1.find_slots(60);
    let slot = slots.next().unwrap();
    assert_eq!(slot.0, Hour::new(14, 30));
    assert_eq!(slot.1, Hour::new(15, 30));
    assert_eq!(slots.next(), None);
}