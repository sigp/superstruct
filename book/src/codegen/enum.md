# Top-level enum

SuperStruct generates an enum that combines all of the generated variant structs.

Consider the `MyStruct` example from the previous page:

```rust,no_run,no_playground
#[superstruct(variants(Foo, Bar))]
struct MyStruct {
    name: String,
    #[superstruct(only(Foo))]
    location: u16,
}
```

The generated enum is:

```rust,no_run,no_playground
enum MyStruct {
    Foo(MyStructFoo),
    Bar(MyStructBar),
}
```

The enum has one variant per variant in `superstruct(variants(..))`, and each
variant contains its generated variant struct. It is named `{BaseName}`.

Generation of the top-level enum can be disabled using the `no_enum` attribute. For more information
see the [Struct attributes](../config/struct.md).

## Getters and setters

The top-level enum has getters and setters for each of the variant fields. They are named:

* `{field_name}()` for getters.
* `{field_name}_mut()` for setters.

If a field is common to all variants, then the getters and setters are _total_ and return `&T`
and `&mut T` respectively, where `T` is the type of the field.

If a field is part of some variants but not others, then the getters and
setters are _partial_ and return `Result<&T, E>` and `Result<&mut T, E>`
respectively.

Many aspects of the getters and setters can be configured, including their
names, whether they `Copy` and which error type `E` is used.
See [Field attributes](../config/field.md).

## Casting methods

The top-level enum has methods to _cast_ it to each of the variants:

* `as_{variantname}` returning `Result<&{VariantStruct}, E>`.
* `as_{variantname}_mut` returning `Result<&mut {VariantStruct}, E>`.

The error type `E` may be controlled by the [`cast_error` attribute](../config/struct.md#cast-error).

## Reference methods

The top-level enum has methods for converting it into the `Ref` and `RefMut` types, which
are described [here](./ref-and-refmut.md).

* `to_ref` returning `{BaseName}Ref`.
* `to_mut` returning `{BaseName}RefMut`.

## `From` implementations

The top-level enum has `From` implementations for converting (owned) variant structs, i.e.

* `impl From<{VariantStruct}> for {BaseName}` for all variants

## Attributes on the enum variants

To add attributes to the enum variants, `enum_variant_attributes` and `specific_enum_variant_attributes`
can be used.

Consider a variant of the `MyStruct` example where you want to derive `serde::Serialize`. However, one
of the fields has a lifetime thus the `#[serde(borrow)]` attribute is required on the enum variants.
In addition, you want to change the name of one of the enum variants when it's serialized:
```rust,no_run,no_playground
#[superstruct(
    variants(Foo, Bar),
    enum_variant_attributes(serde(borrow)),
    specific_enum_variant_attributes(Bar(serde(rename = "Baz"))),
)]
#[derive(serde::Serialize)]
struct MyStruct<'a> {
    name: &'a str,
    #[superstruct(only(Foo))]
    location: u16,
}
```

The generated enum is:

```rust,no_run,no_playground
#[derive(serde::Serialize)]
enum MyStruct<'a> {
    #[serde(borrow)]
    Foo(MyStructFoo<'a>),
    #[serde(borrow, rename = "Baz")]
    Bar(MyStructBar<'a>),
}
```
