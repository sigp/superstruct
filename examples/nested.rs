use superstruct::superstruct;

#[superstruct(variants(A, B), variant_attributes(derive(Debug, Clone)))]
#[derive(Debug, Clone)]
pub struct Inner {
    pub both: &'static str,
    #[superstruct(only(A), partial_getter(copy))]
    pub only_a: &'static str,
}

#[superstruct(variants(A, B), variant_attributes(derive(Debug, Clone)))]
#[derive(Debug, Clone)]
pub struct Outer {
    #[superstruct(only(A), partial_getter(rename = "inner_a"))]
    pub inner: InnerA,
    #[superstruct(only(B), partial_getter(rename = "inner_b"))]
    pub inner: InnerB,
}

impl Outer {
    pub fn inner(&self) -> InnerRef<'_> {
        match self {
            Outer::A(a) => InnerRef::A(&a.inner),
            Outer::B(b) => InnerRef::B(&b.inner),
        }
    }

    pub fn inner_mut(&mut self) -> InnerRefMut<'_> {
        match self {
            Outer::A(a) => InnerRefMut::A(&mut a.inner),
            Outer::B(b) => InnerRefMut::B(&mut b.inner),
        }
    }
}

#[cfg_attr(test, test)]
fn main() {
    let inner_a = InnerA {
        both: "hello",
        only_a: "world",
    };
    let inner_b = InnerB { both: "hello" };

    let mut a = Outer::A(OuterA {
        inner: inner_a.clone(),
    });
    let b = Outer::B(OuterB {
        inner: inner_b.clone(),
    });

    assert_eq!(a.inner_a().unwrap().both, b.inner_b().unwrap().both);
    assert_eq!(a.inner().both(), b.inner().both());
    assert_eq!(a.inner_a().unwrap().only_a, "world");

    *a.inner_mut().only_a_mut().unwrap() = "moon";
    assert_eq!(a.inner().only_a().unwrap(), "moon");
}
