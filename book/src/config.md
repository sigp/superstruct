# Configuration

SuperStruct is a procedural macro, and is configured by `superstruct` attributes on the
type being defined.

* [Struct attributes](./config/struct.md) are applied to the top-level type and configure
  properties relevant to that, as well as defaults for error types.
* [Field attributes](./config/field.md) are applied to each struct field and determine
  the fields of variants, as well as the characteristics of getters and setters.
