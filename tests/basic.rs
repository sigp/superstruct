#![allow(non_local_definitions)] // for macros on structs within test functions

use std::borrow::Cow;
use serde::Deserialize;
use superstruct::superstruct;

#[test]
fn basic() {
    #[superstruct(
        variants(Base, Ext),
        variant_attributes(derive(Debug, PartialEq, Clone)),
        cast_error(ty = "BlockError", expr = "BlockError::WrongVariant"),
        partial_getter_error(ty = "BlockError", expr = "BlockError::WrongVariant")
    )]
    #[derive(Debug, PartialEq, Clone)]
    pub struct Block {
        #[superstruct(getter(copy))]
        slot: u64,
        data: Vec<u8>,
        #[superstruct(only(Ext), partial_getter(copy))]
        description: &'static str,
    }

    #[derive(Debug, PartialEq)]
    pub enum BlockError {
        WrongVariant,
    }

    let base = BlockBase {
        slot: 10,
        data: vec![],
    };
    let ext = BlockExt {
        slot: 11,
        data: vec![10],
        description: "oooeee look at this",
    };

    let mut block1 = Block::Base(base.clone());
    let mut block2 = Block::Ext(ext.clone());

    // Test basic getters.
    assert_eq!(block1.slot(), 10);
    assert_eq!(block2.slot(), 11);

    // Check ref getters.
    let block1_ref = block1.to_ref();
    assert_eq!(block1_ref.slot(), 10);

    // Check casting
    assert_eq!(block1.as_base(), Ok(&base));
    assert_eq!(block1.as_ext(), Err(BlockError::WrongVariant));
    assert_eq!(block2.as_ext(), Ok(&ext));
    assert_eq!(block2.as_base(), Err(BlockError::WrongVariant));

    // Check mutable reference mutators.
    let mut block_mut_ref = block1.to_mut();
    *block_mut_ref.slot_mut() = 1000;
    assert_eq!(block1.slot(), 1000);
    *block1.slot_mut() = 1001;
    assert_eq!(block1.slot(), 1001);

    // Check partial getters.
    assert_eq!(block1.description(), Err(BlockError::WrongVariant));
    assert_eq!(block2.description().unwrap(), ext.description);
    *block2.description_mut().unwrap() = "updated";
    assert_eq!(block2.description().unwrap(), "updated");
}

// Test that superstruct's enum ordering is based on the ordering in `variants(...)`.
// This test fails with variant order (A, B) because A is a subset of B and we're not
// using `serde(deny_unknown_fields)`.
#[test]
fn serde_deserialise_order() {
    #[superstruct(
        variants(B, A),
        variant_attributes(derive(Debug, Deserialize, PartialEq))
    )]
    #[derive(Debug, Deserialize, PartialEq)]
    #[serde(untagged)]
    struct Message {
        common: String,
        #[superstruct(only(B))]
        exclusive: String,
    }

    let message_str = r#"{"common": "hello", "exclusive": "world"}"#;
    let message: Message = serde_json::from_str(message_str).unwrap();

    let expected = Message::B(MessageB {
        common: "hello".into(),
        exclusive: "world".into(),
    });

    assert_eq!(message, expected);
}

#[test]
#[allow(clippy::non_minimal_cfg)]
fn cfg_attribute() {
    // Use `all()` as true.
    #[superstruct(variants(A, B), no_map_macros)]
    struct Message {
        #[cfg(not(all()))]
        pub value: String,
        #[cfg(all())]
        pub value: u64,

        #[superstruct(only(B))]
        #[cfg(not(all()))]
        pub partial: String,
        // Repeating the `only` is somewhat annoying, but OK for now.
        #[superstruct(only(B))]
        #[cfg(all())]
        pub partial: u64,
    }

    let a = Message::A(MessageA { value: 10 });
    assert_eq!(*a.value(), 10);

    let b = Message::B(MessageB {
        value: 10,
        partial: 5,
    });
    assert_eq!(*b.partial().unwrap(), 5);
}

#[test]
fn no_enum() {
    #[superstruct(variants(A, B), no_enum)]
    struct Message {
        #[superstruct(only(A))]
        pub x: u64,
        #[superstruct(only(B))]
        pub y: u64,
    }

    type Message = MessageA;

    let a: Message = Message { x: 0 };
    let b: MessageB = MessageB { y: 0 };
    assert_eq!(a.x, b.y);
}

#[test]
#[allow(dead_code)]
fn no_getter() {
    #[superstruct(variants(A, B))]
    struct NoGetter {
        #[superstruct(only(A), no_getter)]
        pub x: u64,
        #[superstruct(only(B), no_getter)]
        pub x: String,
    }
}

#[test]
#[allow(dead_code)]
fn enum_variant_attribute() {
    #[superstruct(
        variants(A, B),
        variant_attributes(derive(Deserialize)),
        enum_variant_attributes(serde(borrow))
    )]
    #[derive(Deserialize)]
    struct EnumVariantAttribute<'a> {
        #[superstruct(only(A))]
        #[serde(borrow)]
        pub x: Cow<'a, str>,
        #[superstruct(only(B))]
        #[serde(borrow)]
        pub y: Cow<'a, [u8]>,
    }
}
