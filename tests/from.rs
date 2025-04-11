#![allow(non_local_definitions)] // for macros on structs within test functions

use std::fmt::Display;
use superstruct::superstruct;

#[superstruct(
    variants(Good, Bad),
    variant_attributes(derive(Debug, Clone, PartialEq))
)]
#[derive(Debug, Clone, PartialEq)]
pub struct Message<T: Display> {
    #[superstruct(getter(copy))]
    id: u64,
    #[superstruct(only(Good))]
    good: T,
    #[superstruct(only(Bad))]
    bad: T,
}

#[test]
fn generic_from_variant() {
    let message_good_variant = MessageGood {
        id: 0,
        good: "hello",
    };
    let message_bad_variant = MessageBad {
        id: 1,
        bad: "noooooo",
    };

    let message_good = Message::from(message_good_variant);
    let message_bad = Message::from(message_bad_variant);

    assert_eq!(message_good.id(), 0);
    assert_eq!(*message_good.good().unwrap(), "hello");

    assert_eq!(message_bad.id(), 1);
    assert_eq!(*message_bad.bad().unwrap(), "noooooo");
}

#[test]
fn generic_ref_from() {
    let message_good_variant = MessageGood {
        id: 0,
        good: "hello",
    };
    let message_bad_variant = MessageBad {
        id: 1,
        bad: "noooooo",
    };

    // Check Ref from reference to variant.
    let message_good_ref = MessageRef::from(&message_good_variant);
    let message_bad_ref = MessageRef::from(&message_bad_variant);

    assert_eq!(message_good_ref.id(), 0);
    assert_eq!(*message_good_ref.good().unwrap(), "hello");

    assert_eq!(message_bad_ref.id(), 1);
    assert_eq!(*message_bad_ref.bad().unwrap(), "noooooo");

    // Check Ref from reference to top-level enum.
    let message_good = Message::from(message_good_variant.clone());
    let message_good_ref = MessageRef::from(&message_good);

    assert_eq!(message_good_ref.id(), 0);
    assert_eq!(*message_good_ref.good().unwrap(), "hello");
}
