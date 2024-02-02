# Meta variant structs and enums

Meta variants are an optional feature, useful for scenarios where you'd want nested
enums at the top-level. structs will be created for all combinations of `meta_variants`
and `variants`, names in the format `{BaseName}{MetaVariantName}{VariantName}`. 
Additionally, enums will be created for each `meta_variant` named `{BaseName}{MetaVariantName}`.

For example:

```rust,no_run,no_playground
#[superstruct(meta_variants(Baz, Qux), variants(Foo, Bar))]
struct MyStruct {
    name: String,
    #[superstruct(only(Foo))]
    location: u16,
    #[superstruct(meta_only(Baz))]
    score: u64,
    #[superstruct(only(Bar), meta_only(Qux))]
    id: usize,
}
```

Here the `BaseName` is `MyStruct` and there are two variants in the meta-enum called 
`Baz` and `Qux`.

The generated enums are:

```rust,no_run,no_playground
enum MyStruct{
    Baz(MyStructBaz),
    Qux(MyStructQux),
}

enum MyStructBaz{
    Foo(MyStructBazFoo),
    Bar(MyStructBazBar),
}

enum MyStructQux{
    Foo(MyStructQuxFoo),
    Bar(MyStructQuxBar),
}
```

The generated variant structs are:

```rust,no_run,no_playground
struct MyStructBazFoo {
    name: String,
    location: u16,
    score: u64,
}

struct MyStructBazBar {
    name: String,
    score: u64,
}

struct MyStructQuxFoo {
    name: String,
    location: u16,
}

struct MyStructQuxBar {
    name: String,
    id: usize,
}
```

Note how the `only` attribute still applies, and a new `meta_only` attribute can be used to
control the presence of fields in each meta variant.

For more information see [Struct attributes](../config/struct.md).
