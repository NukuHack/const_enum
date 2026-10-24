//! Laws of `VariantName` over arbitrary inputs.
//! On failure proptest prints the minimal failing input and stores its seed in `proptest-regressions/`.
#![allow(dead_code)]

use const_enum::VariantName;
use proptest::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, VariantName)]
enum Pred {
	Always,
	EnemyHpLt,
	SelfIntegrityLt,
	#[name = "custom_name"]
	#[alias = "odd"]
	#[alias = "weird_one"]
	SomethingOdd,
	#[name = "x1"]
	Digit,
}

const ACCEPTED: [(&str, Pred); 7] = [
	("always", Pred::Always),
	("enemy_hp_lt", Pred::EnemyHpLt),
	("self_integrity_lt", Pred::SelfIntegrityLt),
	("custom_name", Pred::SomethingOdd),
	("odd", Pred::SomethingOdd),
	("weird_one", Pred::SomethingOdd),
	("x1", Pred::Digit),
];

fn expected(s: &str) -> Option<Pred> {
	ACCEPTED.iter().find(|(name, _)| *name == s).map(|(_, p)| *p)
}

/// Flips the ASCII case of the characters selected by `mask` (bit `i % 64` for character `i`).
fn flip_case(s: &str, mask: u64) -> String {
	s.chars()
		.enumerate()
		.map(|(i, c)| {
			if mask >> (i % 64) & 1 == 1 {
				if c.is_ascii_lowercase() {
					c.to_ascii_uppercase()
				} else {
					c.to_ascii_lowercase()
				}
			} else {
				c
			}
		})
		.collect()
}

proptest! {
	#[test]
	fn parse_accepts_exactly_the_declared_spellings(s in ".*") {
		prop_assert_eq!(Pred::parse(&s), expected(&s));
	}

	#[test]
	fn parse_accepts_exactly_the_declared_spellings_for_near_misses(
		index in 0..ACCEPTED.len(),
		cut in 0usize..20,
		extra in "[a-z_ 0-9]{0,3}",
	) {
		let (name, _) = ACCEPTED[index];
		let truncated: String = name.chars().take(cut).collect();
		let extended = format!("{name}{extra}");
		prop_assert_eq!(Pred::parse(&truncated), expected(&truncated));
		prop_assert_eq!(Pred::parse(&extended), expected(&extended));
	}

	#[test]
	fn ignore_case_accepts_any_ascii_case_variation(index in 0..ACCEPTED.len(), mask in any::<u64>()) {
		let (name, variant) = ACCEPTED[index];
		prop_assert_eq!(Pred::parse_ignore_ascii_case(&flip_case(name, mask)), Some(variant));
	}

	#[test]
	fn exact_parse_rejects_any_case_variation_that_changes_the_text(index in 0..ACCEPTED.len(), mask in any::<u64>()) {
		let (name, _) = ACCEPTED[index];
		let flipped = flip_case(name, mask);
		prop_assume!(flipped != name);
		prop_assert_eq!(Pred::parse(&flipped), None);
	}

	#[test]
	fn ignore_case_matches_a_reference_implementation(s in ".{0,24}") {
		let reference = ACCEPTED.iter().find(|(name, _)| name.eq_ignore_ascii_case(&s)).map(|(_, p)| *p);
		prop_assert_eq!(Pred::parse_ignore_ascii_case(&s), reference);
	}

	#[test]
	fn exact_parse_implies_ignore_case_parse(s in ".{0,24}") {
		if let Some(v) = Pred::parse(&s) {
			prop_assert_eq!(Pred::parse_ignore_ascii_case(&s), Some(v));
		}
	}

	#[test]
	fn from_index_succeeds_exactly_below_count(i in any::<usize>()) {
		prop_assert_eq!(Pred::from_index(i).is_some(), i < Pred::COUNT);
	}

	#[test]
	fn from_index_round_trips_through_index(i in 0..Pred::COUNT) {
		let v = Pred::from_index(i).expect("in range");
		prop_assert_eq!(v.index(), i);
		prop_assert_eq!(Pred::ALL[i], v);
		prop_assert_eq!(Pred::NAMES[i], v.name());
	}

	#[test]
	fn display_and_name_always_agree(i in 0..Pred::COUNT) {
		let v = Pred::ALL[i];
		prop_assert_eq!(v.to_string(), v.name());
	}
}

#[test]
fn flip_case_helper_is_an_involution_on_ascii() {
	for (name, _) in ACCEPTED {
		for mask in [0, 1, u64::MAX, 0xAAAA_AAAA_AAAA_AAAA] {
			assert_eq!(flip_case(&flip_case(name, mask), mask), name);
		}
	}
}
