//! Derive macros for enum <-> primitive and enum <-> string conversions that work in `const` contexts.
//! Trait methods cannot be `const` on stable, so everything is generated as inherent `const fn`s
//! with thin, non-const std trait impls (`From`, `TryFrom`, `Display`) on top.
//! Each derive lives in its own module; shared validation helpers are in `validate`.

mod case;
mod primitive;
mod validate;
mod variant_name;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use syn::DeriveInput;

fn expand(input: TokenStream, expander: fn(&DeriveInput) -> syn::Result<TokenStream2>) -> TokenStream {
	syn::parse::<DeriveInput>(input).and_then(|input| expander(&input)).unwrap_or_else(syn::Error::into_compile_error).into()
}

/// Derives const conversions between a fieldless enum and its `#[repr(int)]` type.
///
/// Discriminants may be explicit, implicit or any const expression: the macro asks the
/// compiler via `Variant as repr` instead of evaluating them itself. Without a `#[repr]`
/// the type is `isize`, like rustc.
///
/// Generated: `to_primitive`, `try_from_primitive`, `from_primitive_or`, `From<Enum> for repr`
/// and `TryFrom<repr> for Enum` (error = the rejected number). `from_primitive` is only
/// generated when a variant is marked `#[default]`.
///
/// ```
/// use const_enum::ConstFromPrimitive;
///
/// #[derive(Clone, Copy, Debug, Default, PartialEq, ConstFromPrimitive)]
/// #[repr(u8)]
/// enum Op {
///     #[default]
///     Nop = 0,
///     Add,
///     Sub = 10,
/// }
///
/// const A: Op = Op::from_primitive(10);
/// const B: Option<Op> = Op::try_from_primitive(7);
/// assert_eq!(A, Op::Sub);
/// assert_eq!(B, None);
/// ```
#[proc_macro_derive(ConstFromPrimitive, attributes(default))]
pub fn derive_const_from_primitive(input: TokenStream) -> TokenStream {
	expand(input, primitive::expand)
}

/// Derives string conversions and variant tables for a fieldless enum.
///
/// A variant's string is its identifier converted to snake_case (`EnemyHpPct` -> `"enemy_hp_pct"`).
/// Customize with:
///
/// * `#[name = "x"]` on a variant: explicit string
/// * `#[alias = "y"]` on a variant (repeatable): extra strings accepted by `parse`
/// * `#[name_case = "kebab-case"]` on the enum: one of `snake_case`, `kebab-case`,
///   `SCREAMING_SNAKE_CASE`, `SCREAMING-KEBAB-CASE`, `lowercase`, `UPPERCASE`, `camelCase`, `PascalCase`
///
/// Generated (all `const`): `COUNT`, `ALL`, `NAMES`, `name`, `index`, `from_index`, `parse`,
/// `parse_ignore_ascii_case`; plus `Display`.
///
/// ```
/// use const_enum::VariantName;
///
/// #[derive(Clone, Copy, Debug, PartialEq, VariantName)]
/// enum Pred {
///     Always,
///     #[alias = "hp_lt"]
///     EnemyHpLt,
/// }
///
/// const P: Option<Pred> = Pred::parse("hp_lt");
/// assert_eq!(P, Some(Pred::EnemyHpLt));
/// assert_eq!(Pred::EnemyHpLt.name(), "enemy_hp_lt");
/// ```
#[proc_macro_derive(VariantName, attributes(name, alias, name_case))]
pub fn derive_variant_name(input: TokenStream) -> TokenStream {
	expand(input, variant_name::expand)
}
