use superstruct::superstruct;

#[superstruct(
    meta_variants(Read, Write),
    variants(Lower, Upper),
    variant_attributes(derive(Clone, Debug, PartialEq, Eq))
)]
#[derive(Clone, Debug, PartialEq, Eq)]
struct InnerMessage {
    // Exists on all structs.
    pub w: u64,
    // Exists on all Read structs.
    #[superstruct(meta_only(Read))]
    pub x: u64,
    // Exists on all LowerCase structs.
    #[superstruct(only(Lower))]
    pub y: u64,
    // Exists only in InnerMessageWriteLower.
    #[superstruct(meta_only(Write), only(Upper))]
    pub z: u64,
}

#[test]
fn meta_variant() {
    let message_a = InnerMessage::Read(InnerMessageRead::Lower(InnerMessageReadLower {
        w: 1,
        x: 2,
        y: 3,
    }));
    assert_eq!(*message_a.w(), 1);
    assert_eq!(*message_a.x().unwrap(), 2);
    assert_eq!(*message_a.y().unwrap(), 3);
    assert!(message_a.z().is_err());

    let message_b = InnerMessage::Read(InnerMessageRead::Upper(InnerMessageReadUpper {
        w: 1,
        x: 2,
    }));
    assert_eq!(*message_b.w(), 1);
    assert_eq!(*message_b.x().unwrap(), 2);
    assert!(message_b.y().is_err());
    assert!(message_b.z().is_err());

    let message_c = InnerMessage::Write(InnerMessageWrite::Lower(InnerMessageWriteLower {
        w: 1,
        y: 3,
    }));
    assert_eq!(*message_c.w(), 1);
    assert!(message_c.x().is_err());
    assert_eq!(*message_c.y().unwrap(), 3);
    assert!(message_c.z().is_err());

    let message_d = InnerMessage::Write(InnerMessageWrite::Upper(InnerMessageWriteUpper {
        w: 1,
        z: 4,
    }));
    assert_eq!(*message_d.w(), 1);
    assert!(message_d.x().is_err());
    assert!(message_d.y().is_err());
    assert_eq!(*message_d.z().unwrap(), 4);
}

#[superstruct(
    meta_variants(Read, Write),
    variants(Lower, Upper),
    variant_attributes(derive(Debug, PartialEq, Eq))
)]
#[derive(Debug, PartialEq, Eq)]
struct Message {
    // Exists on all variants.
    #[superstruct(flatten)]
    pub inner_a: InnerMessage,
    // Exists on all Upper variants.
    #[superstruct(flatten(Upper))]
    pub inner_b: InnerMessage,
    // Exists on all Read variants.
    #[superstruct(flatten(Read))]
    pub inner_c: InnerMessage,
    // Exists on only the Read + Lower variant.
    #[superstruct(flatten(Write, Lower))]
    pub inner_d: InnerMessage,
}

#[test]
fn meta_variant_flatten() {
    let inner_a = InnerMessageReadLower { w: 1, x: 2, y: 3 };
    let inner_c = InnerMessageReadLower { w: 4, x: 5, y: 6 };
    let message_e = Message::Read(MessageRead::Lower(MessageReadLower { inner_a, inner_c }));
    assert_eq!(message_e.inner_a_read_lower().unwrap().w, 1);
    assert!(message_e.inner_a_read_upper().is_err());
    assert!(message_e.inner_a_write_lower().is_err());
    assert!(message_e.inner_a_write_upper().is_err());

    assert_eq!(message_e.inner_c_read_lower().unwrap().w, 4);
    assert!(message_e.inner_c_read_upper().is_err());

    let inner_a = InnerMessageReadUpper { w: 1, x: 2 };
    let inner_b = InnerMessageReadUpper { w: 3, x: 4 };
    let inner_c = InnerMessageReadUpper { w: 5, x: 6 };
    let message_f = Message::Read(MessageRead::Upper(MessageReadUpper {
        inner_a,
        inner_b,
        inner_c,
    }));
    assert!(message_f.inner_a_read_lower().is_err());
    assert_eq!(message_f.inner_a_read_upper().unwrap().w, 1);
    assert!(message_f.inner_a_write_lower().is_err());
    assert!(message_f.inner_a_write_upper().is_err());

    assert_eq!(message_f.inner_b_read_upper().unwrap().w, 3);
    assert!(message_f.inner_b_write_upper().is_err());

    assert!(message_f.inner_c_read_lower().is_err());
    assert_eq!(message_f.inner_c_read_upper().unwrap().w, 5);

    let inner_a = InnerMessageWriteLower { w: 1, y: 2 };
    let inner_d = InnerMessageWriteLower { w: 3, y: 4 };
    let message_g = Message::Write(MessageWrite::Lower(MessageWriteLower { inner_a, inner_d }));
    assert!(message_g.inner_a_read_lower().is_err());
    assert!(message_g.inner_a_read_upper().is_err());
    assert_eq!(message_g.inner_a_write_lower().unwrap().w, 1);
    assert!(message_g.inner_a_write_upper().is_err());

    assert_eq!(message_g.inner_d_write_lower().unwrap().w, 3);

    let inner_a = InnerMessageWriteUpper { w: 1, z: 2 };
    let inner_b = InnerMessageWriteUpper { w: 3, z: 4 };
    let message_h = Message::Write(MessageWrite::Upper(MessageWriteUpper { inner_a, inner_b }));
    assert!(message_h.inner_a_read_lower().is_err());
    assert!(message_h.inner_a_read_upper().is_err());
    assert!(message_h.inner_a_write_lower().is_err());
    assert_eq!(message_h.inner_a_write_upper().unwrap().w, 1);

    assert!(message_h.inner_b_read_upper().is_err());
    assert_eq!(message_h.inner_b_write_upper().unwrap().w, 3);
}
