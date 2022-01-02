# Field attributes

Field attributes may be applied to fields within a `struct` that has a `superstruct` attribute
to it at the top-level.

All attributes are optional.

## Only

```
#[superstruct(only(A, B, ...))]
```

Define the list of variants that this field is a member of.

The `only` attribute is currently the only way that different variants are
created.

The selected variants should be a subset of the variants defined in the top-level
[`variants`](./struct.md#variants) attribute.

**Format**: 1+ comma-separated identifiers.

## Getter

```
#[superstruct(getter(copy, ..))]
#[superstruct(getter(no_mut, ..))]
#[superstruct(getter(rename = "..", ..))]
```

Customise the implementation of the [getter functions](../codegen/enum.md#getters-and-setters) for
this field.

This attribute can only be applied to **common** fields (i.e. ones with no `only` attribute).

All of the sub-attributes `copy`, `no_mut` and `rename` are optional and any subset of them
may be applied in a single attribute, e.g. `#[superstruct(getter(copy, no_mut))]` is valid.

* `copy`: return `T` rather than `&T` where `T` is the type of the field. `T` must be `Copy`
  or the generated code will fail to typecheck.
* `no_mut`: do not generate a mutating getter with `_mut` suffix.
* `rename = "name"`: rename the immutable getter to `name()` and the mutable getter to `name_mut()`
  (if enabled).

## Partial getter

```
#[superstruct(partial_getter(copy, ..))]
#[superstruct(partial_getter(no_mut, ..))]
#[superstruct(partial_getter(rename = "..", ..))]
```

Customise the implementation of the [partial getter
functions](../codegen/enum.md#getters-and-setters) for this field.

This attribute can only be applied to **_non_-common** fields (i.e. ones _with_ an `only` attribute).

All of the sub-attributes `copy`, `no_mut` and `rename` are optional and any subset of them
may be applied in a single attribute, e.g. `#[superstruct(partial_getter(copy, no_mut))]` is valid.

* `copy`: return `Result<T, E>` rather than `Result<&T, E>` where `T` is the type of the field. `T`
  must be `Copy` or the generated code will fail to typecheck.
* `no_mut`: do not generate a mutating getter with `_mut` suffix.
* `rename = "name"`: rename the immutable partial getter to `name()` and the mutable partial getter
  to `name_mut()` (if enabled).

The error type for partial getters can currently only be configured on a per-struct basis
via the [`partial_getter_error`](./struct.md#partial-getter-error) attribute, although this may
change in a future release.
