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
    #[serde(untagged)]
    #[derive(Debug, Deserialize, PartialEq)]
    struct Message {
        common: String,
        #[superstruct(only(B))]
        exclusive: String,
    }

    let message_str = r#"{"common": "hello", "exclusive": "world"}"#;
    let message: Message = serde_json::from_str(&message_str).unwrap();

    let expected = Message::B(MessageB {
        common: "hello".into(),
        exclusive: "world".into(),
    });

    assert_eq!(message, expected);
}
