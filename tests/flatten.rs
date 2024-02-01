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

#[test]
fn flatten_subset() {
    #[superstruct(variants(A, B), variant_attributes(derive(Debug, PartialEq, Eq)))]
    #[derive(Debug, PartialEq, Eq)]
    struct InnerMessageSubset {
        pub x: u64,
        #[superstruct(only(B))]
        pub y: u64,
    }

    #[superstruct(variants(A, B, C), variant_attributes(derive(Debug, PartialEq, Eq)))]
    #[derive(Debug, PartialEq, Eq)]
    struct MessageSubset {
        #[superstruct(flatten(A, B))]
        pub inner: InnerMessageSubset,
    }

    let message_a = MessageSubset::A(MessageSubsetA {
        inner: InnerMessageSubsetA { x: 1 },
    });
    let message_b = MessageSubset::B(MessageSubsetB {
        inner: InnerMessageSubsetB { x: 3, y: 4 },
    });
    let message_c = MessageSubset::C(MessageSubsetC {});
    assert_eq!(message_a.inner_a().unwrap().x, 1);
    assert!(message_a.inner_b().is_err());
    assert_eq!(message_b.inner_b().unwrap().x, 3);
    assert_eq!(message_b.inner_b().unwrap().y, 4);
    assert!(message_b.inner_a().is_err());
    assert!(message_c.inner_a().is_err());
    assert!(message_c.inner_b().is_err());
}

#[test]
fn flatten_not_first_field() {
    use test_mod::*;

    // Put this type in a submodule to test path parsing in `flatten`.
    pub mod test_mod {
        use superstruct::superstruct;

        #[superstruct(variants(A, B), variant_attributes(derive(Debug, PartialEq, Eq)))]
        #[derive(Debug, PartialEq, Eq)]
        pub struct InnerMessageTwo {
            pub x: u64,
            #[superstruct(only(B))]
            pub y: u64,
        }
    }

    #[superstruct(variants(A, B), variant_attributes(derive(Debug, PartialEq, Eq)))]
    #[derive(Debug, PartialEq, Eq)]
    struct MessageTwo {
        #[superstruct(only(A), partial_getter(copy))]
        pub other: u64,
        #[superstruct(flatten)]
        pub inner: test_mod::InnerMessageTwo,
    }

    let message_a = MessageTwo::A(MessageTwoA {
        other: 21,
        inner: InnerMessageTwoA { x: 1 },
    });
    let message_b = MessageTwo::B(MessageTwoB {
        inner: InnerMessageTwoB { x: 3, y: 4 },
    });
    assert_eq!(message_a.other().unwrap(), 21);
    assert_eq!(message_a.inner_a().unwrap().x, 1);
    assert!(message_a.inner_b().is_err());
    assert_eq!(message_b.inner_b().unwrap().x, 3);
    assert_eq!(message_b.inner_b().unwrap().y, 4);
    assert!(message_b.inner_a().is_err());

    let message_a_ref = MessageTwoRef::A(&MessageTwoA {
        other: 21,
        inner: InnerMessageTwoA { x: 1 },
    });
    let message_b_ref = MessageTwoRef::B(&MessageTwoB {
        inner: InnerMessageTwoB { x: 3, y: 4 },
    });
    assert_eq!(message_a.other().unwrap(), 21);
    assert_eq!(message_a_ref.inner_a().unwrap().x, 1);
    assert!(message_a_ref.inner_b().is_err());
    assert_eq!(message_b_ref.inner_b().unwrap().x, 3);
    assert_eq!(message_b_ref.inner_b().unwrap().y, 4);
    assert!(message_b_ref.inner_a().is_err());

    let mut inner_a = MessageTwoA {
        other: 21,
        inner: InnerMessageTwoA { x: 1 },
    };
    let mut inner_b = MessageTwoB {
        inner: InnerMessageTwoB { x: 3, y: 4 },
    };

    // Re-initialize the struct to avoid borrow checker errors.
    let mut message_a_ref_mut = MessageTwoRefMut::A(&mut inner_a);
    assert_eq!(message_a_ref_mut.inner_a_mut().map(|inner| inner.x), Ok(1));
    let mut message_a_ref_mut = MessageTwoRefMut::A(&mut inner_a);
    assert!(message_a_ref_mut.inner_b_mut().is_err());
    let mut message_b_ref_mut = MessageTwoRefMut::B(&mut inner_b);
    assert_eq!(message_b_ref_mut.inner_b_mut().unwrap().x, 3);
    let mut message_b_ref_mut = MessageTwoRefMut::B(&mut inner_b);
    assert_eq!(message_b_ref_mut.inner_b_mut().unwrap().y, 4);
    let mut message_b_ref_mut = MessageTwoRefMut::B(&mut inner_b);
    assert!(message_b_ref_mut.inner_a_mut().is_err());
}
