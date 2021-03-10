use superstruct::superstruct;

#[superstruct(variants(Base, Lmao))]
struct Block {
    slot: u64,
    #[superstruct(only(Lmao))]
    lmao: &'static str,
}

#[test]
fn wow() {
    let base = BlockBase { slot: 10 };
    let lmao = BlockLmao {
        slot: 10,
        lmao: "holy shit",
    };

    let block1 = Block::Base(base);
    let block2 = Block::Lmao(lmao);
}
