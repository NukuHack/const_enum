//! One test per bug found in an earlier version of the macros; each fails on the old behavior.
#![allow(dead_code)]

use const_enum::{ConstFromPrimitive, VariantName};

// Old: a variant without `= N` made the macro panic at compile time.
#[derive(Clone, Copy, Debug, Default, PartialEq, ConstFromPrimitive)]
#[repr(u8)]
enum ImplicitDiscriminants {
	#[default]
	A,
	B,
	C = 9,
	D,
}

#[test]
fn variants_without_explicit_discriminants_are_supported() {
	assert_eq!(ImplicitDiscriminants::from_primitive(1), ImplicitDiscriminants::B);
	assert_eq!(ImplicitDiscriminants::from_primitive(10), ImplicitDiscriminants::D);
	assert_eq!(ImplicitDiscriminants::from_primitive(2), ImplicitDiscriminants::A);
}

// Old: only the first argument of the first `#[repr]` was read, so a leading hint hid the integer type
// and the macro silently fell back to `i32`.
#[derive(Clone, Copy, Debug, PartialEq, ConstFromPrimitive)]
#[repr(align(4), u8)]
enum LeadingHint {
	A = 200,
}

#[test]
fn a_hint_before_the_integer_type_does_not_hide_it() {
	let n: u8 = LeadingHint::A.to_primitive();
	assert_eq!(n, 200);
	assert_eq!(LeadingHint::try_from_primitive(200), Some(LeadingHint::A));
}

// Old: the fallback repr was `i32`; rustc's own default for discriminants is `isize`.
#[derive(Clone, Copy, Debug, PartialEq, ConstFromPrimitive)]
enum DefaultRepr {
	A = 1,
}

#[test]
fn the_fallback_repr_is_isize() {
	let n: isize = DefaultRepr::A.to_primitive();
	assert_eq!(n, 1);
}

// Old: `#[default]` was only accepted next to `#[derive(Default)]`.
#[derive(Debug, PartialEq, ConstFromPrimitive)]
#[repr(u8)]
enum DefaultWithoutDeriveDefault {
	#[default]
	A = 0,
	B = 1,
}

#[test]
fn default_attribute_works_without_deriving_default() {
	assert_eq!(DefaultWithoutDeriveDefault::from_primitive(1), DefaultWithoutDeriveDefault::B);
	assert_eq!(DefaultWithoutDeriveDefault::from_primitive(7), DefaultWithoutDeriveDefault::A);
}

// Old: a missing `#[default]` was a hard error even if only the `Option`-returning API was wanted.
#[derive(Debug, PartialEq, ConstFromPrimitive)]
#[repr(u8)]
enum NoDefault {
	A = 0,
}

#[test]
fn default_attribute_is_optional() {
	assert_eq!(NoDefault::try_from_primitive(0), Some(NoDefault::A));
	assert_eq!(NoDefault::try_from_primitive(1), None);
}

// Old: `parse` used `match` on `&str`, which is not allowed in `const fn` on stable compilers.
#[derive(Clone, Copy, Debug, PartialEq, VariantName)]
enum ConstParse {
	One,
	Two,
}

const PARSED: Option<ConstParse> = ConstParse::parse("two");

#[test]
fn parse_is_evaluated_at_compile_time() {
	assert_eq!(PARSED, Some(ConstParse::Two));
}

// Old: a literal forwarded through `macro_rules!` is wrapped in an invisible group and was rejected.
macro_rules! renamed {
	($enum:ident, $name:literal, $alias:literal, $case:literal) => {
		#[derive(Clone, Copy, Debug, PartialEq, VariantName)]
		#[name_case = $case]
		enum $enum {
			#[name = $name]
			#[alias = $alias]
			A,
			B,
		}
	};
}

renamed!(FromMacro, "from_macro", "macro_alias", "kebab-case");

#[test]
fn attribute_literals_forwarded_by_macro_rules_are_accepted() {
	assert_eq!(FromMacro::A.name(), "from_macro");
	assert_eq!(FromMacro::parse("macro_alias"), Some(FromMacro::A));
	assert_eq!(FromMacro::B.name(), "b");
}
