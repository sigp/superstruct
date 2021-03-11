use superstruct::superstruct;

#[superstruct(variants(Base, Ext), derive_all(Debug))]
#[derive(Debug)]
pub struct Block {
    #[superstruct(getter(copy))]
    slot: u64,
    #[superstruct(only(Ext))]
    description: &'static str,
}

#[test]
fn wow() {
    let base = BlockBase { slot: 10 };
    let lmao = BlockExt {
        slot: 11,
        description: "oooeee look at this",
    };

    let block1 = Block::Base(base);
    let block2 = Block::Ext(lmao);

    println!("{:?}", block1);
    println!("{:?}", block2);
    println!("{}", block1.slot());
}
