use std::fmt::Display;
use superstruct::superstruct;

#[superstruct(variants(Good, Bad), variant_attributes(derive(Debug, PartialEq)))]
#[derive(Debug, PartialEq)]
pub struct Message<T: Display> {
    #[superstruct(getter(copy))]
    id: u64,
    #[superstruct(only(Good))]
    good: T,
    #[superstruct(only(Bad))]
    bad: T,
}

#[test]
fn generic_from() {
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
