# `Ref` and `RefMut`

SuperStruct generates two reference-like structs which are designed to simplify working with nested
`superstruct` types.

The immutable reference type is named `{BaseName}Ref` and has all of the immutable getter methods
from the top-level enum.

The mutable reference type is named `{BaseName}RefMut` and has all of the mutable getter methods
from the top-level enum.

Consider the `MyStruct` example again:

```rust,no_run,no_playground
#[superstruct(variants(Foo, Bar))]
struct MyStruct {
    name: String,
    #[superstruct(only(Foo))]
    location: u16,
}
```

The generated `Ref` types look like this:

```rust,no_run,no_playground
enum MyStructRef<'a> {
    Foo(&'a MyStructFoo),
    Bar(&'a MyStructBar),
}

enum MyStructRefMut<'a> {
    Foo(&'a mut MyStructFoo),
    Bar(&'a mut MyStructFoo),
}
```

The reason these types can be useful (particularly with nesting) is that they do not require a full
reference to a `MyStruct` in order to construct: a reference to a single variant struct will suffice.

## Trait Implementations

### `Copy`

Each `Ref` type is `Copy`, just like an ordinary `&T`.

### `From`

The `Ref` type has `From` implementations that allow converting from references to variants
or references to the top-level enum type, i.e.

- `impl From<&'a {VariantStruct}> for {BaseName}Ref<'a>` for all variants.
- `impl From<&'a {BaseName}> for {BaseName}Ref<'a>` (same as `to_ref()`).

## Example

Please see [`examples/nested.rs`](../rustdoc/src/nested/nested.rs.html) and its
generated [documentation](../rustdoc/nested/).
