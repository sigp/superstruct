use superstruct::*;

#[superstruct(
    variants(A, B),
    variant_attributes(cfg_attr(all(), derive(PartialEq, Debug)))
)]
#[cfg_attr(all(), derive(PartialEq, Debug))]
struct Thing {
    x: u64,
}

#[test]
fn thing_impls_partial_eq() {
    let t1 = Thing::A(ThingA { x: 5 });
    assert_eq!(t1, t1);
}
