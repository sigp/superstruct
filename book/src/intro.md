# Introduction

SuperStruct is a Rust library for working with versioned data. It allows you to define and operate
on _variants_ of a `struct` which share some fields in common.

As an example, imagine you're working on a program that accepts a `Request` struct from the user.
In the first version of the program you only allow users to specify a `start: u16` field:

```rust,no_run,no_playground
pub struct Request {
    start: u16,
}
```

After a while you realise that it would be nice if users could also specify an `end: u16` in their
requests, so you would like to change the definition of `Request` to:

```rust,no_run,no_playground
pub struct Request {
    start: u16,
    end: u16,
}
```

Now imagine that your program needs to work with old versions of `Request` as well as new, i.e.
it needs to be backwards-compatible. This is reasonably common when databases are involved and
you need to write schema migrations, or when working with network protocols.

SuperStruct allows you to define _both_ versions of the `Request` with a single definition, and
also generates an enum to unify them:

```rust,no_run,noplayground
{{#include ../../examples/request.rs}}
```

The `superstruct` definition generates:

* Two structs `RequestV1` and `RequestV2` where the `end` field is only present in `RequestV2`.
* An enum `Request` with variants `V1` and `V2` wrapping `RequestV1` and `RequestV2` respectively.
* A getter function on `Request` for the shared `start` field, e.g. `r1.start()`.
* A _partial_ getter function returning `Result<&u16, ()>` for `end`, e.g. `r2.end()`.
* Lots of other useful goodies that are covered in the [Codegen](./codegen.md) section of the book.

## When _should_ you use SuperStruct?

* If you want to avoid duplication when defining multiple related structs.
* If you are considering manually writing getters to extract common fields from an enum.
* If you are considering writing traits to unify types with fields in common.

## When should you _not_ use SuperStruct?

* If you can get away with just using an `Option` field. In our example, `Request` could define
  `end: Option<u16>`.
* If you can achieve backwards compatible (de)serialization through clever use of `serde` macros.

## What next?

* Check out the [Code Generation](./codegen.md) docs.
* Check out the [Configuration](./config.md) docs for information on how to
  control `superstruct`'s behaviour, including renaming getters, working with
  `Copy` types, etc.
