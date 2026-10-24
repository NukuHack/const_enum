//! Contract of the default `VariantName` behavior (snake_case names, no attributes):
//! * `ALL`, `NAMES`, `COUNT` list every variant in declaration order;
//! * `name(v)` is the snake_case identifier and `parse(name(v)) == Some(v)`;
//! * `parse` is exact (case, whitespace and prefixes matter), `parse_ignore_ascii_case` only folds ASCII case;
//! * `index`/`from_index` are inverse and independent of the discriminant;
//! * everything is `const`, `Display` prints `name()`.
#![allow(dead_code)]

use const_enum::{ConstFromPrimitive, VariantName};

#[derive(Clone, Copy, Debug, PartialEq, Eq, VariantName)]
enum Pred {
	Always,
	EnemyHpLt,
	SelfIntegrityLt,
	SelfCharge,
}

const PRED_NAMES: [(Pred, &str); 4] =
	[(Pred::Always, "always"), (Pred::EnemyHpLt, "enemy_hp_lt"), (Pred::SelfIntegrityLt, "self_integrity_lt"), (Pred::SelfCharge, "self_charge")];

#[test]
fn name_is_the_snake_case_identifier() {
	for (pred, name) in PRED_NAMES {
		assert_eq!(pred.name(), name);
	}
}

#[test]
fn tables_list_every_variant_in_declaration_order() {
	assert_eq!(Pred::COUNT, 4);
	assert_eq!(Pred::ALL, [Pred::Always, Pred::EnemyHpLt, Pred::SelfIntegrityLt, Pred::SelfCharge]);
	assert_eq!(Pred::NAMES, ["always", "enemy_hp_lt", "self_integrity_lt", "self_charge"]);
	assert_eq!(Pred::ALL.len(), Pred::COUNT);
	assert_eq!(Pred::NAMES.len(), Pred::COUNT);
}

#[test]
fn parse_inverts_name_for_every_variant() {
	for pred in Pred::ALL {
		assert_eq!(Pred::parse(pred.name()), Some(pred));
		assert_eq!(Pred::parse_ignore_ascii_case(pred.name()), Some(pred));
	}
}

#[test]
fn parse_is_exact() {
	for bad in [
		"",
		" ",
		"Always",
		"ALWAYS",
		"always ",
		" always",
		"always\n",
		"alway",
		"alwayss",
		"enemy-hp-lt",
		"enemyhplt",
		"EnemyHpLt",
		"enemy_hp_lt_",
		"_enemy_hp_lt",
		"enemy_hp",
		"always\0",
	] {
		assert_eq!(Pred::parse(bad), None, "{bad:?}");
	}
}

#[test]
fn parse_does_not_confuse_names_sharing_a_prefix() {
	assert_eq!(Pred::parse("self_charge"), Some(Pred::SelfCharge));
	assert_eq!(Pred::parse("self_integrity_lt"), Some(Pred::SelfIntegrityLt));
	assert_eq!(Pred::parse("self_"), None);
	assert_eq!(Pred::parse("self"), None);
}

#[test]
fn ignore_case_folds_ascii_letters_only() {
	assert_eq!(Pred::parse_ignore_ascii_case("ALWAYS"), Some(Pred::Always));
	assert_eq!(Pred::parse_ignore_ascii_case("Enemy_Hp_Lt"), Some(Pred::EnemyHpLt));
	assert_eq!(Pred::parse_ignore_ascii_case("sElF_cHaRgE"), Some(Pred::SelfCharge));
	assert_eq!(Pred::parse_ignore_ascii_case("always "), None);
	assert_eq!(Pred::parse_ignore_ascii_case(""), None);
	assert_eq!(Pred::parse_ignore_ascii_case("enemy-hp-lt"), None, "separators are not folded");
	assert_eq!(Pred::parse_ignore_ascii_case("alwayſ"), None, "non-ASCII lookalikes are not folded");
}

#[test]
fn index_and_from_index_are_inverse() {
	for (i, pred) in Pred::ALL.into_iter().enumerate() {
		assert_eq!(pred.index(), i);
		assert_eq!(Pred::from_index(i), Some(pred));
	}
}

#[test]
fn from_index_rejects_out_of_range() {
	assert_eq!(Pred::from_index(Pred::COUNT), None);
	assert_eq!(Pred::from_index(Pred::COUNT + 1), None);
	assert_eq!(Pred::from_index(usize::MAX), None);
}

#[test]
fn display_prints_the_name() {
	for (pred, name) in PRED_NAMES {
		assert_eq!(pred.to_string(), name);
		assert_eq!(format!("{pred}"), name);
	}
}

const C_NAME: &str = Pred::SelfCharge.name();
const C_INDEX: usize = Pred::EnemyHpLt.index();
const C_FROM_INDEX: Option<Pred> = Pred::from_index(2);
const C_PARSE: Option<Pred> = Pred::parse("enemy_hp_lt");
const C_PARSE_NONE: Option<Pred> = Pred::parse("nope");
const C_PARSE_CI: Option<Pred> = Pred::parse_ignore_ascii_case("ALWAYS");
const C_FIRST: Pred = Pred::ALL[0];
static S_NAMES: [&str; Pred::COUNT] = Pred::NAMES;
const LEN_ARRAY: [u8; Pred::COUNT] = [0; Pred::COUNT];

#[test]
fn everything_is_const_evaluable() {
	assert_eq!(C_NAME, "self_charge");
	assert_eq!(C_INDEX, 1);
	assert_eq!(C_FROM_INDEX, Some(Pred::SelfIntegrityLt));
	assert_eq!(C_PARSE, Some(Pred::EnemyHpLt));
	assert_eq!(C_PARSE_NONE, None);
	assert_eq!(C_PARSE_CI, Some(Pred::Always));
	assert_eq!(C_FIRST, Pred::Always);
	assert_eq!(S_NAMES[3], "self_charge");
	assert_eq!(LEN_ARRAY.len(), 4);
}

const fn parse_index(s: &str) -> Option<usize> {
	match Pred::parse(s) {
		Some(p) => Some(p.index()),
		None => None,
	}
}

#[test]
fn usable_inside_const_fn_and_const_asserts() {
	const _: () = assert!(matches!(parse_index("self_charge"), Some(3)));
	const _: () = assert!(parse_index("zzz").is_none());
	const _: () = assert!(Pred::COUNT == 4);
}

// Declaration order and discriminant order deliberately disagree.
#[derive(Clone, Copy, Debug, PartialEq, ConstFromPrimitive, VariantName)]
#[repr(u8)]
enum Mixed {
	Last = 30,
	First = 10,
	Middle = 20,
}

#[test]
fn index_follows_declaration_order_not_the_discriminant() {
	assert_eq!(Mixed::Last.index(), 0);
	assert_eq!(Mixed::First.index(), 1);
	assert_eq!(Mixed::Middle.index(), 2);
	assert_eq!(Mixed::ALL, [Mixed::Last, Mixed::First, Mixed::Middle]);
	assert_eq!(Mixed::from_index(1), Some(Mixed::First));
	assert_eq!(Mixed::try_from_primitive(10), Some(Mixed::First));
}

#[test]
fn both_derives_coexist_without_clashes() {
	for v in Mixed::ALL {
		assert_eq!(Mixed::parse(v.name()), Some(v));
		assert_eq!(Mixed::try_from_primitive(v.to_primitive()), Some(v));
	}
}

#[derive(Debug, PartialEq, VariantName)]
enum SingleVariant {
	Only,
}

#[test]
fn single_variant_enum() {
	assert_eq!(SingleVariant::COUNT, 1);
	assert_eq!(SingleVariant::parse("only"), Some(SingleVariant::Only));
	assert_eq!(SingleVariant::parse("other"), None);
	assert_eq!(SingleVariant::from_index(0), Some(SingleVariant::Only));
	assert_eq!(SingleVariant::from_index(1), None);
}

// Deliberately neither `Clone` nor `Copy`.
#[derive(Debug, PartialEq, VariantName)]
enum NotCopy {
	A,
	B,
}

#[test]
fn enums_that_are_not_copy_are_supported() {
	let a = NotCopy::A;
	assert_eq!(a.name(), "a");
	assert_eq!(a.index(), 0);
	assert_eq!(NotCopy::parse("b"), Some(NotCopy::B));
	assert_eq!(NotCopy::B.to_string(), "b");
	assert_eq!(NotCopy::NAMES, ["a", "b"]);
}

#[derive(Clone, Copy, Debug, PartialEq, VariantName)]
enum PreludeNames {
	Some,
	None,
	Ok,
	Err,
	Option,
	Result,
}

#[test]
fn variants_named_like_prelude_items_do_not_confuse_the_expansion() {
	for v in PreludeNames::ALL {
		assert_eq!(PreludeNames::parse(v.name()), Some(v));
	}
	assert_eq!(PreludeNames::parse("none"), Some(PreludeNames::None));
	assert_eq!(PreludeNames::from_index(0), Some(PreludeNames::Some));
}

#[test]
fn enums_declared_inside_a_function_are_supported() {
	#[derive(Clone, Copy, Debug, PartialEq, VariantName)]
	enum Local {
		FooBar,
		Baz,
	}

	assert_eq!(Local::parse("foo_bar"), Some(Local::FooBar));
	assert_eq!(Local::Baz.name(), "baz");
}

mod visibility {
	use const_enum::VariantName;

	/// Doc comments and foreign attributes on the enum and its variants must be ignored.
	#[derive(Clone, Copy, Debug, PartialEq, VariantName)]
	#[allow(dead_code)]
	pub(crate) enum Documented {
		/// First.
		#[allow(unused)]
		FooBar,
		/// Second.
		#[doc(hidden)]
		Baz,
	}

	#[test]
	fn doc_comments_and_foreign_attributes_are_ignored() {
		assert_eq!(Documented::parse("foo_bar"), Some(Documented::FooBar));
		assert_eq!(Documented::NAMES, ["foo_bar", "baz"]);
	}
}

#[derive(Clone, Copy, Debug, PartialEq, VariantName)]
enum ManyVariants {
	V00,
	V01,
	V02,
	V03,
	V04,
	V05,
	V06,
	V07,
	V08,
	V09,
	V10,
	V11,
	V12,
	V13,
	V14,
	V15,
	V16,
	V17,
	V18,
	V19,
	V20,
	V21,
	V22,
	V23,
	V24,
	V25,
	V26,
	V27,
	V28,
	V29,
}

#[test]
fn many_variants_all_round_trip() {
	assert_eq!(ManyVariants::COUNT, 30);
	for (i, v) in ManyVariants::ALL.into_iter().enumerate() {
		assert_eq!(v.index(), i);
		assert_eq!(ManyVariants::parse(v.name()), Some(v));
		assert_eq!(ManyVariants::parse_ignore_ascii_case(&v.name().to_uppercase()), Some(v));
	}
	assert_eq!(ManyVariants::parse("v30"), None);
	assert_eq!(ManyVariants::parse("v0"), None);
}
