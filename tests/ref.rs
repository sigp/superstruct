use superstruct::superstruct;

// Check that we can convert a Ref to an inner reference with the same lifetime as `message`.
#[test]
fn getter_and_partial_getter_lifetimes() {
    #[superstruct(variants(A, B))]
    struct Message {
        pub x: String,
        #[superstruct(only(B))]
        pub y: String,
    }

    fn get_x(message: &Message) -> &String {
        message.to_ref().x()
    }

    fn get_y(message: &Message) -> Result<&String, ()> {
        message.to_ref().y()
    }

    let m = Message::B(MessageB {
        x: "hello".into(),
        y: "world".into(),
    });
    let x = get_x(&m);
    let y = get_y(&m).unwrap();
    assert_eq!(x, "hello");
    assert_eq!(y, "world");
}
