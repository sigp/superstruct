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

#[test]
fn map_into() {
    #[superstruct(
        variants(A, B),
        variant_attributes(derive(Debug, PartialEq, Clone)),
        map_into(Thing2),
        map_ref_into(Thing2Ref),
        map_ref_mut_into(Thing2RefMut)
    )]
    #[derive(Debug, PartialEq, Clone)]
    pub struct Thing1 {
        #[superstruct(only(A), partial_getter(rename = "thing2a"))]
        thing2: Thing2A,
        #[superstruct(only(B), partial_getter(rename = "thing2b"))]
        thing2: Thing2B,
    }

    #[superstruct(variants(A, B), variant_attributes(derive(Debug, PartialEq, Clone)))]
    #[derive(Debug, PartialEq, Clone)]
    pub struct Thing2 {
        x: u64,
    }

    fn thing1_to_thing2(thing1: Thing1) -> Thing2 {
        map_thing1_into_thing2!(thing1, |inner, cons| { cons(inner.thing2) })
    }

    fn thing1_ref_to_thing2_ref<'a>(thing1: Thing1Ref<'a>) -> Thing2Ref<'a> {
        map_thing1_ref_into_thing2_ref!(&'a _, thing1, |inner, cons| { cons(&inner.thing2) })
    }

    fn thing1_ref_mut_to_thing2_ref_mut<'a>(thing1: Thing1RefMut<'a>) -> Thing2RefMut<'a> {
        map_thing1_ref_mut_into_thing2_ref_mut!(&'a _, thing1, |inner, cons| {
            cons(&mut inner.thing2)
        })
    }

    let thing2a = Thing2A { x: 10 };
    let mut thing2 = Thing2::A(thing2a.clone());
    let mut thing1 = Thing1::A(Thing1A { thing2: thing2a });
    assert_eq!(thing1_to_thing2(thing1.clone()).x(), thing2.x());
    assert_eq!(
        thing1_ref_to_thing2_ref(thing1.to_ref()).x(),
        thing2.to_ref().x()
    );
    assert_eq!(
        thing1_ref_mut_to_thing2_ref_mut(thing1.to_mut()).x_mut(),
        thing2.to_mut().x_mut()
    );

    // Mutatating through the Thing2RefMut should change the value.
    *thing1_ref_mut_to_thing2_ref_mut(thing1.to_mut()).x_mut() = 11;
    assert_eq!(*thing1_ref_to_thing2_ref(thing1.to_ref()).x(), 11);
}
