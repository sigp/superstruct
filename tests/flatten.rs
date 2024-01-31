use superstruct::superstruct;

#[test]
fn flatten() {
    #[superstruct(variants(A, B), variant_attributes(derive(Debug, PartialEq, Eq)))]
    #[derive(Debug, PartialEq, Eq)]
    struct InnerMessage {
        pub x: u64,
        #[superstruct(only(B))]
        pub y: u64,
    }

    #[superstruct(variants(A, B), variant_attributes(derive(Debug, PartialEq, Eq)))]
    #[derive(Debug, PartialEq, Eq)]
    struct Message {
        #[superstruct(flatten)]
        pub inner: InnerMessage,
    }

    let message_a = Message::A(MessageA {
        inner: InnerMessageA { x: 1 },
    });
    let message_b = Message::B(MessageB {
        inner: InnerMessageB { x: 3, y: 4 },
    });
    assert_eq!(message_a.inner_a().unwrap().x, 1);
    assert!(message_a.inner_b().is_err());
    assert_eq!(message_b.inner_b().unwrap().x, 3);
    assert_eq!(message_b.inner_b().unwrap().y, 4);
    assert!(message_b.inner_a().is_err());

    let message_a_ref = MessageRef::A(&MessageA {
        inner: InnerMessageA { x: 1 },
    });
    let message_b_ref = MessageRef::B(&MessageB {
        inner: InnerMessageB { x: 3, y: 4 },
    });
    assert_eq!(message_a_ref.inner_a().unwrap().x, 1);
    assert!(message_a_ref.inner_b().is_err());
    assert_eq!(message_b_ref.inner_b().unwrap().x, 3);
    assert_eq!(message_b_ref.inner_b().unwrap().y, 4);
    assert!(message_b_ref.inner_a().is_err());

    let mut inner_a = MessageA {
        inner: InnerMessageA { x: 1 },
    };
    let mut inner_b = MessageB {
        inner: InnerMessageB { x: 3, y: 4 },
    };

    // Re-initialize the struct to avoid borrow checker errors.
    let mut message_a_ref_mut = MessageRefMut::A(&mut inner_a);
    assert_eq!(message_a_ref_mut.inner_a_mut().map(|inner| inner.x), Ok(1));
    let mut message_a_ref_mut = MessageRefMut::A(&mut inner_a);
    assert!(message_a_ref_mut.inner_b_mut().is_err());
    let mut message_b_ref_mut = MessageRefMut::B(&mut inner_b);
    assert_eq!(message_b_ref_mut.inner_b_mut().unwrap().x, 3);
    let mut message_b_ref_mut = MessageRefMut::B(&mut inner_b);
    assert_eq!(message_b_ref_mut.inner_b_mut().unwrap().y, 4);
    let mut message_b_ref_mut = MessageRefMut::B(&mut inner_b);
    assert!(message_b_ref_mut.inner_a_mut().is_err());
}
