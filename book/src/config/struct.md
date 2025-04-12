# Struct attributes

The following attributes may be used in a `superstruct` macro invocation on a
`struct` item. All attributes are optional unless stated otherwise.

## Variants

```
#[superstruct(variants(A, B, ...))]
```

Define the list of variants that this type has.
See [variant structs](../codegen/variant-structs.md).

The `variants` attribute is _not optional_.

**Format**: 1+ comma-separated identifiers.

## Cast error

```
#[superstruct(cast_error(ty = "..", expr = ".."))]
```

Define the error type to be returned from [casting methods](../codegen/enum.md#casting-methods).

The expression must be of the given error type, and capable of being evaluated without any
context (it is _not_ a closure).

**Format**: quoted type for `ty`, quoted expression for `expr`

## Partial getter error

```
#[superstruct(cast_error(ty = "..", expr = ".."))]
```

Define the error type to be returned from [partial getter
methods](../codegen/enum.md#getters-and-setters).

The expression must be of the given error type, and capable of being evaluated without any
context (it is _not_ a closure).

**Format**: quoted type for `ty`, quoted expression for `expr`

## Variant attributes

```
#[superstruct(variant_attributes(...))]
```

Provide a list of attributes to be applied verbatim to each variant struct definition.

This can be used to derive traits, perform conditional compilation, etc.

**Format**: any.

## Specific variant attributes

```
#[superstruct(specific_variant_attributes(A(...), B(...), ...))]
```

Similar to `variant_attributes`, but applies the attributes _only_ to the named variants. This
is useful if e.g. one variant needs to derive a trait which the others cannot, or if another
procedural macro is being invoked on the variant struct which requires different parameters.

**Format**: zero or more variant names, with variant attributes nested in parens

## Enum variant attributes

```
#[superstruct(enum_variant_attributes(...))]
```

Provide a list of attributes to be applied verbatim to each of the enum variants.

This is useful when using another proc-macro on the enum and needing to add an attribute
to all enum variants.

**Format**: any.

## `Ref` attributes

```
#[superstruct(ref_attributes(...))]
```

Provide a list of attributes to be applied verbatim to the generated `Ref` type.

**Format**: any.

## `RefMut` attributes

```
#[superstruct(ref_mut_attributes(...))]
```

Provide a list of attributes to be applied verbatim to the generated `RefMut` type.

**Format**: any.

## No enum

```
#[superstruct(no_enum)]
```

Disable generation of the top-level enum, and all code except the
[variant structs](../codegen/variant-structs.md).

## Map Into

```
#[map_into(ty1, ty2, ..)]
#[map_ref_into(ty1, ty2, ..)]
#[map_ref_mut_into(ty1, ty2, ..)]
```

Generate mapping macros from the top-level enum, the `Ref` type or the `RefMut` type as appropriate.

Please see the documentation on [Mapping into other types](./codegen/map-macros.md#mapping-into-other-types)
for an explanation of how these macros operate.

**Format**: one or more `superstruct` type names

## Meta variants

```
#[superstruct(meta_variants(A, B, ...), variants(C, D, ...))]
```

Generate a two-dimensional superstruct.
See [meta variant structs](../codegen/meta-variants.md).

The `meta_variants` attribute is optional.

**Format**: 1+ comma-separated identifiers.
