use superstruct::superstruct;

#[test]
fn partial_getter() {
    #[superstruct(variants(A, B))]
    struct Message {
        pub x: u64,
        #[superstruct(only(B))]
        pub y: u64,
    }

    let mut m = Message::B(MessageB { x: 0, y: 10 });
    let mut mut_ref = m.to_mut();
    *mut_ref.y_mut().unwrap() = 100;

    assert_eq!(*m.y().unwrap(), 100);
    assert_eq!(*m.x(), 0);
}

#[test]
fn copy_partial_getter() {
    #[superstruct(variants(A, B))]
    struct Message {
        #[superstruct(getter(copy))]
        pub x: u64,
        #[superstruct(only(B), partial_getter(copy))]
        pub y: u64,
    }

    let mut m = Message::B(MessageB { x: 0, y: 10 });
    let mut mut_ref = m.to_mut();
    *mut_ref.y_mut().unwrap() = 100;

    assert_eq!(m.y().unwrap(), 100);
    assert_eq!(m.x(), 0);
}
