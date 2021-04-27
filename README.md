SuperStruct
===========

![test status](https://github.com/sigp/superstruct/actions/workflows/test-suite.yml/badge.svg)

SuperStruct is a library for working with a family of related struct _variants_, where each variant shares some common fields, and adds in unique fields of its own.

Its design is informed by the implementation of blockchain consensus upgrades, which often change core data structures by removing some old fields and replacing them with new ones.

Currently the library is unstable and evolving rapidly alongside [Lighthouse][] as it becomes hard-fork aware.

You can run `cargo expand --test basic` to see the code generated for `tests/basic.rs`.

[Lighthouse]: https://github.com/sigp/lighthouse
