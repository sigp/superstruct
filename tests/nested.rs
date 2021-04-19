use superstruct::superstruct;

#[superstruct(variants(A, B), variant_attributes(derive(Debug, Clone)))]
#[derive(Debug, Clone)]
struct Inner {
    both: &'static str,
    #[superstruct(only(A))]
    only_a: &'static str,
}

#[superstruct(variants(A, B), variant_attributes(derive(Debug, Clone)))]
#[derive(Debug, Clone)]
struct Outer {
    #[superstruct(only(A), partial_getter(rename = "inner_a"))]
    inner: InnerA,
    #[superstruct(only(B), partial_getter(rename = "inner_b"))]
    inner: InnerB,
}

impl Outer {
    pub fn inner(&self) -> InnerRef<'_> {
        match self {
            Outer::A(a) => InnerRef::A(&a.inner),
            Outer::B(b) => InnerRef::B(&b.inner),
        }
    }
}

#[test]
fn nesting() {
    let inner_a = InnerA {
        both: "hello",
        only_a: "world",
    };
    let inner_b = InnerB { both: "hello" };

    let a = Outer::A(OuterA {
        inner: inner_a.clone(),
    });
    let b = Outer::B(OuterB {
        inner: inner_b.clone(),
    });

    assert_eq!(a.inner_a().unwrap().both, b.inner_b().unwrap().both);
    assert_eq!(a.inner().both(), b.inner().both());
}
