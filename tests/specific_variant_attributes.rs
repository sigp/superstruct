use superstruct::superstruct;

#[superstruct(
    variants(IsCopy, IsNotCopy),
    variant_attributes(derive(Debug, PartialEq, Clone)),
    specific_variant_attributes(IsCopy(derive(Copy)))
)]
#[derive(Clone, PartialEq, Debug)]
pub struct Thing {
    pub x: u64,
    #[superstruct(only(IsNotCopy))]
    pub y: String,
}

#[test]
fn copy_the_thing() {
    fn copy<T: Copy>(t: T) -> (T, T) {
        (t, t)
    }

    let x = ThingIsCopy { x: 0 };
    assert_eq!(copy(x), (x, x));
}
