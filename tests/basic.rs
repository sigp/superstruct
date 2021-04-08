use superstruct::superstruct;

#[superstruct(
    variants(Base, Ext),
    variant_attributes(derive(Debug, PartialEq)),
    cast_error(ty = "BlockError", expr = "BlockError::WrongVariant")
)]
#[derive(Debug, PartialEq)]
pub struct Block {
    #[superstruct(getter(copy))]
    slot: u64,
    data: Vec<u8>,
    #[superstruct(only(Ext))]
    description: &'static str,
}

pub enum BlockError {
    WrongVariant,
}

#[test]
fn wow() {
    let base = BlockBase {
        slot: 10,
        data: vec![],
    };
    let lmao = BlockExt {
        slot: 11,
        data: vec![10],
        description: "oooeee look at this",
    };

    let mut block1 = Block::Base(base);
    let block2 = Block::Ext(lmao);

    println!("{:?}", block1);
    println!("{:?}", block2);
    println!("{}", block1.slot());

    let block_ref = block1.to_ref();
    println!("{:?}", block_ref.slot());

    let mut block_mut_ref = block1.to_mut();
    println!("{:?}", block_mut_ref.slot_mut());
}
