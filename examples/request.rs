use superstruct::superstruct;

#[superstruct(variants(V1, V2))]
pub struct Request {
    pub start: u16,
    #[superstruct(only(V2))]
    pub end: u16,
}

#[cfg_attr(test, test)]
fn main() {
    let r1 = Request::V1(RequestV1 { start: 0 });
    let r2 = Request::V2(RequestV2 { start: 0, end: 10 });

    assert_eq!(r1.start(), r2.start());
    assert_eq!(r1.end(), Err(()));
    assert_eq!(r2.end(), Ok(&10));
}
