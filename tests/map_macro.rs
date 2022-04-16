use superstruct::superstruct;

#[test]
fn map_macro_basic() {
    #[superstruct(variants(Base, Ext), variant_attributes(derive(Debug, PartialEq)))]
    #[derive(Debug, PartialEq)]
    pub struct Block {
        #[superstruct(getter(copy))]
        slot: u64,
        #[superstruct(only(Ext), partial_getter(copy))]
        description: &'static str,
    }

    fn increment_slot(block: Block) -> Block {
        map_block!(block, |mut inner, cons| {
            inner.slot += 1;
            cons(inner)
        })
    }

    fn get_slot_via_ref<'a>(block_ref: BlockRef<'a>) -> u64 {
        map_block_ref!(&'a _, block_ref, |inner, _| { inner.slot })
    }

    assert_eq!(
        increment_slot(Block::Base(BlockBase { slot: 10 })).slot(),
        get_slot_via_ref(Block::Base(BlockBase { slot: 11 }).to_ref())
    );
    assert_eq!(
        increment_slot(Block::Ext(BlockExt {
            slot: 0,
            description: "test"
        })),
        Block::Ext(BlockExt {
            slot: 1,
            description: "test"
        })
    );
}

#[test]
fn map_macro_generic() {
    #[superstruct(variants(Base, Ext), variant_attributes(derive(Debug, PartialEq)))]
    #[derive(Debug, PartialEq)]
    pub struct Blob<T> {
        slot: T,
        #[superstruct(only(Ext), partial_getter(copy))]
        description: &'static str,
    }

    impl From<BlobBase<u8>> for BlobBase<u16> {
        fn from(blob: BlobBase<u8>) -> Self {
            BlobBase {
                slot: blob.slot as u16,
            }
        }
    }

    impl From<BlobExt<u8>> for BlobExt<u16> {
        fn from(blob: BlobExt<u8>) -> Self {
            Self {
                slot: blob.slot as u16,
                description: blob.description,
            }
        }
    }

    impl From<Blob<u8>> for Blob<u16> {
        fn from(blob: Blob<u8>) -> Self {
            map_blob!(blob, |inner, cons| { cons(inner.into()) })
        }
    }

    assert_eq!(
        Blob::Base(BlobBase { slot: 10u16 }),
        Blob::Base(BlobBase { slot: 10u8 }).into(),
    );
}
