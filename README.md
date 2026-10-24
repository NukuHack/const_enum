# const_enum

Derive macros for enum ⇄ primitive and enum ⇄ string conversions where **everything is a `const fn`**
(usable in `const`, `static`, array lengths, `match` arms, …). Generated code is `no_std`-compatible.

Trait methods can't be `const` on stable Rust, so all the const functionality is generated as
**inherent methods**; the standard traits (`From`, `TryFrom`, `Display`) are thin non-const wrappers.

The macro crate builds on Rust 1.70+. The test suite needs a recent stable compiler
(`proptest`, and `#[repr(u128)]`/`#[repr(i128)]` in the 128-bit tests).

## Compile / test

```sh
cargo build                  # build the macro crate
cargo test                   # unit, integration, property and doc tests
cargo fmt                    # formatting, configured in rustfmt.toml
cargo clippy --all-targets   # lints
```

Use it from another crate:

```toml
[dependencies]
const_enum = { path = "../const_enum" }   # or a git/registry dependency
```

To see what the macros generate: `cargo install cargo-expand`, then `cargo expand --test variant_name_basic`.

## Usage

```rust
use const_enum::{ConstFromPrimitive, VariantName};

const BASE: u8 = 100;

#[derive(Clone, Copy, Debug, Default, PartialEq, ConstFromPrimitive, VariantName)]
#[repr(u8)]
#[name_case = "kebab-case"]            // optional, default is snake_case
enum Op {
    #[default]                          // optional, enables `from_primitive`
    Nop = 0,
    Add,                                // implicit discriminants work (= 1)
    #[name = "sub"]                     // explicit string
    #[alias = "minus"]                  // extra accepted strings for parse
    Sub = 10,
    FooBar = BASE + 5,                  // any const expression works
}

// primitive <-> enum
const A: Op = Op::from_primitive(10);                 // Op::Sub (falls back to #[default] if unknown)
const B: Option<Op> = Op::try_from_primitive(7);      // None
const C: Op = Op::from_primitive_or(7, Op::Add);      // Op::Add
const D: u8 = Op::FooBar.to_primitive();              // 105
let e: Result<Op, u8> = Op::try_from(1u8);            // Ok(Op::Add); Err(n) hands the number back
let n: u8 = Op::Sub.into();                           // 10

// string <-> enum
const NAME: &str = Op::FooBar.name();                           // "foo-bar"
const P: Option<Op> = Op::parse("minus");                       // Some(Op::Sub)
const Q: Option<Op> = Op::parse_ignore_ascii_case("FOO-BAR");   // Some(Op::FooBar)
static NAMES: [&str; Op::COUNT] = Op::NAMES;                    // ["nop", "add", "sub", "foo-bar"]
assert_eq!(Op::Add.to_string(), "add");                         // Display

// tables
for op in Op::ALL { println!("{} = {}", op.name(), op.to_primitive()); }
assert_eq!(Op::from_index(2), Some(Op::Sub));
assert_eq!(Op::Sub.index(), 2);                                 // declaration order, not the discriminant
```

### `ConstFromPrimitive`

| item | notes |
|---|---|
| `to_primitive(self) -> repr` | |
| `try_from_primitive(n) -> Option<Self>` | |
| `from_primitive_or(n, fallback) -> Self` | |
| `from_primitive(n) -> Self` | only generated if a variant has `#[default]` |
| `From<Enum> for repr`, `TryFrom<repr> for Enum` | `Error` is the rejected number |

The integer type is taken from `#[repr(u8)]` etc., also next to other hints such as
`#[repr(align(4), u8)]` or in a separate `#[repr(...)]` attribute. Without an integer `repr` it is
`isize`, like rustc. `#[default]` is registered by the derive, so `#[derive(Default)]` is not required;
marking more than one variant is a compile error.

### `VariantName`

| item | notes |
|---|---|
| `COUNT`, `ALL: [Self; N]`, `NAMES: [&str; N]` | declaration order |
| `name(&self)`, `index(&self)`, `from_index(i)` | |
| `parse(s)` | exact; a `match` on the string's bytes (`match` on `&str` is still unstable in `const fn`) |
| `parse_ignore_ascii_case(s)` | ASCII case-insensitive, linear scan |
| `Display` | prints `name()` |

Attributes: `#[name = ".."]`, `#[alias = ".."]` (repeatable) on variants; `#[name_case = ".."]` on the enum
(`snake_case`, `kebab-case`, `SCREAMING_SNAKE_CASE`, `SCREAMING-KEBAB-CASE`, `lowercase`, `UPPERCASE`,
`camelCase`, `PascalCase`). `name_case` only affects generated names, not `#[name]`/`#[alias]` values.
Duplicate names/aliases (after case conversion too) are compile errors.

### Limitations

Only non-generic enums with unit variants (and at least one variant) are supported.
`FromStr` is not generated (it would need a shared error type).

## Tests

| file | area |
|---|---|
| `src/*.rs` (`#[cfg(test)]`) | white-box: case splitting, `#[repr]` parsing, validation and attribute error messages (private to the proc-macro crate) |
| `tests/primitive_basic.rs` | `from_primitive` for every repr, sign, sparse/expression discriminants |
| `tests/primitive_api.rs` | `try_from_primitive`, `from_primitive_or`, `to_primitive`, `From`/`TryFrom`, const use, every integer repr at its limits |
| `tests/primitive_props.rs` | property-based laws and an exhaustive `u16` sweep |
| `tests/variant_name_basic.rs` | names, tables, `parse`, `index`, `Display`, const use |
| `tests/variant_name_attrs.rs` | `name`, `alias`, all eight `name_case` styles, raw/non-ASCII identifiers |
| `tests/variant_name_props.rs` | property-based parse/ignore-case/index laws against reference implementations |
| `tests/regressions.rs` | one test per bug fixed since the first version |

`proptest` is a dev-dependency only: the parse/convert laws hold over a huge input space and shrinking
makes failures readable. On failure it prints the minimal input and writes its seed to `proptest-regressions/`.
